use std::vec;

use crate::{
    empty_row, header_row, impl_text_view,
    model::{
        champion::Champion,
        game::{ChampSelectSession, GameState, LiveGameSession, PlayerInfo, PostGameSession},
        ids::ChampionId,
        mastery::Mastery,
        summoner::SummonerWithStats,
    },
    service::lookup::LookupService,
    styled_line, styled_span,
    ui::{
        async_data::AsyncData, views::RenderableView, Controller, RenderContext, TextCreationResult, ViewError,
        ViewResult,
    },
};
use itertools::Itertools;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

// ============================================================================
// Champion Select Info View
// ============================================================================

type ChampionSelectAramEntry = (Champion, Option<Mastery>);

fn get_champ_info(champ: &ChampionId, lookup: &LookupService) -> Result<ChampionSelectAramEntry, ViewError> {
    let champion = lookup.get_champion(champ)?;
    let mastery = match champion.owned {
        true => lookup.get_mastery(champ).ok(),
        false => None,
    };

    Ok((champion.clone(), mastery))
}

fn format_selectable_champ(entry: ChampionSelectAramEntry) -> Result<String, ViewError> {
    let (champion, mastery) = entry;
    let mut output = format!("  {:<16}", format!("{}:", champion.name));
    match champion.owned {
        true => match mastery {
            Some(mastery) => {
                output.push_str(&format!("  Level {}", mastery.level));
                if mastery.level > 5 {
                    output.push_str(&format!(
                        " ({} pts, {}/{} marks)",
                        mastery.points, mastery.marks, mastery.required_marks
                    ));
                } else {
                    output.push_str(&format!(" ({} pts)", mastery.points));
                }
            }
            None => output.push_str("  Level 0 (not played!)"),
        },
        false => output.push_str("  not owned!"),
    }
    Ok(output)
}

fn get_entries(champ_ids: &[ChampionId], lookup: &LookupService) -> Result<Vec<ChampionSelectAramEntry>, ViewError> {
    let mut entries = champ_ids
        .iter()
        .filter(|champ| champ.0 != "0")
        .map(|champ| get_champ_info(champ, lookup))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|(champ, mastery)| {
        (
            !champ.owned,
            mastery
                .clone()
                .map_or((0, 0), |m| (-(m.level as i32), -(m.points as i32))),
        )
    });
    Ok(entries)
}

fn get_team_selections(session: &ChampSelectSession) -> (ChampionId, Vec<ChampionId>) {
    let mut team_champs = session
        .my_team
        .iter()
        .map(|t| t.selected_champion.clone())
        .collect_vec();

    let local_player = session.local_player_cell;
    if let Some(pos) = session.my_team.iter().position(|t| t.cell_id == local_player) {
        let current_champ = team_champs.swap_remove(pos);
        (current_champ, team_champs)
    } else {
        (ChampionId("0".into()), team_champs)
    }
}

fn champ_select_aram_view(ctrl: &Controller) -> TextCreationResult {
    let mut lines = Vec::new();

    match ctrl.manager.get_champ_select().recv().unwrap()? {
        Some(champ_select_info) => {
            let queue = ctrl.lookup.get_queue(champ_select_info.queue_id)?;

            match queue.pick_mode.as_str() {
                "AllRandomPickStrategy" => {
                    let (current_champ, team_champs) = get_team_selections(&champ_select_info);

                    lines.push(styled_line!("Currently selected champ:"; Color::Rgb(200, 150, 0)));
                    if current_champ == ChampionId("0".into()) {
                        lines.push(styled_line!("  Not yet selected"; Color::LightBlue));
                    } else {
                        lines.push(styled_line!(
                            "{}",
                            format_selectable_champ(get_champ_info(&current_champ, ctrl.lookup)?)?
                        ));
                    }

                    lines.push(styled_line!());
                    lines.push(styled_line!("Benched Champions:"; Color::Rgb(200, 150, 0)));
                    let benched = get_entries(&champ_select_info.benched_champs, ctrl.lookup)?;

                    for entry in benched {
                        lines.push(styled_line!("{}", format_selectable_champ(entry)?));
                    }

                    lines.push(styled_line!());
                    lines.push(styled_line!("Tradable Champions:"; Color::Rgb(200, 150, 0)));

                    let team = get_entries(&team_champs, ctrl.lookup)?;
                    for entry in team {
                        lines.push(styled_line!("{}", format_selectable_champ(entry)?));
                    }
                }
                _ => lines.extend(vec![
                    styled_line!(),
                    styled_line!("  This view only supports ARAM champ selects, and this is {:?}.", queue; Color::Yellow),
                ]),
            }
        }
        None => lines.extend(vec![styled_line!(), styled_line!("  Not in champ select!"; Color::Red)]),
    };
    Ok(lines)
}

impl_text_view!(ChampSelectAramView, champ_select_aram_view, "ARAM Champ Select Info", auto_refresh: 0.5);

// ===========================================================================
//   Future Game Info Views
// ==========================================================================

pub struct LivePlayerInfoView {
    cs_data: Option<AsyncData<Option<ChampSelectSession>>>,
    live_game_data: Option<AsyncData<Option<LiveGameSession>>>,
    post_game_data: Option<AsyncData<Option<PostGameSession>>>,
    players_data: Option<AsyncData<Vec<SummonerWithStats>>>,
    game_state: Option<GameState>,
    self_info: (String, String),
}

impl LivePlayerInfoView {
    pub fn new(ctrl: &Controller) -> Self {
        let summoner = &ctrl.manager.get_summoner();

        let mut view = Self {
            cs_data: None,
            live_game_data: None,
            post_game_data: None,
            game_state: None,
            players_data: None,
            self_info: (summoner.game_name.clone(), summoner.tag_line.clone()),
        };
        view.start_session_requests(ctrl);
        view
    }

    fn start_session_requests(&mut self, ctrl: &Controller) {
        // Start all three fetches simultaneously
        let cs_rx = ctrl.manager.get_champ_select();
        let live_rx = ctrl.manager.get_live_game();
        let post_rx = ctrl.manager.get_post_game();

        self.cs_data = Some(AsyncData::new(cs_rx));
        self.live_game_data = Some(AsyncData::new(live_rx));
        self.post_game_data = Some(AsyncData::new(post_rx));
        self.players_data = None;
    }

    fn columns(&self) -> [Constraint; 9] {
        [
            Constraint::Length(6),  // Team
            Constraint::Length(10), // Position
            Constraint::Length(30), // Player Name
            Constraint::Length(6),  // Level
            Constraint::Length(14), // Queue Type
            Constraint::Length(12), // Rank
            Constraint::Length(5),  // LP
            Constraint::Length(20), // Wins / Losses
            Constraint::Length(25), // Mastery
        ]
    }

    fn format_position(position: &str) -> &str {
        match position.to_uppercase().as_str() {
            "TOP" => "Top",
            "JUNGLE" => "Jungle",
            "MIDDLE" => "Mid",
            "BOTTOM" => "Bot",
            "UTILITY" => "Support",
            _ => "Fill",
        }
    }

    fn position_sort_key(position: &str) -> u8 {
        match position.to_lowercase().as_str() {
            "top" => 0,
            "jungle" => 1,
            "middle" => 2,
            "bottom" => 3,
            "utility" => 4,
            _ => 5,
        }
    }

    fn get_rank_color(tier: &str) -> Color {
        match tier.to_uppercase().as_str() {
            "IRON" => Color::Rgb(107, 104, 102),
            "BRONZE" => Color::Rgb(173, 113, 74),
            "SILVER" => Color::Rgb(181, 192, 196),
            "GOLD" => Color::Rgb(255, 215, 0),
            "PLATINUM" => Color::Rgb(77, 166, 160),
            "EMERALD" => Color::Rgb(34, 197, 94),
            "DIAMOND" => Color::Rgb(147, 197, 253),
            "MASTER" => Color::Rgb(168, 85, 247),
            "GRANDMASTER" => Color::Rgb(239, 68, 68),
            "CHALLENGER" => Color::Rgb(251, 191, 36),
            _ => Color::White,
        }
    }

    fn format_queue_type(queue: &str) -> &str {
        match queue {
            "RANKED_SOLO_5x5" => "Solo/Duo",
            "RANKED_FLEX_SR" => "Flex",
            _ => queue,
        }
    }

    fn format_rank(tier: &str, division: &str) -> String {
        if tier.is_empty() {
            "Unranked".to_string()
        } else {
            let tier_cap = tier.chars().next().unwrap().to_uppercase().collect::<String>() + &tier[1..].to_lowercase();
            format!("{} {}", tier_cap, division)
        }
    }

    fn format_champion_name(champion: &Option<String>) -> ratatui::text::Line<'_> {
        match champion {
            Some(champ) => styled_line!(champ.clone(); Color::White),
            None => styled_line!("Unknown"; Color::DarkGray),
        }
    }

    fn format_mastery(level: u16, points: u32) -> String {
        format!(
            "{} pts (Lvl {})",
            points
                .to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(std::str::from_utf8)
                .collect::<Result<Vec<_>, _>>()
                .map(|parts| parts.join(","))
                .unwrap(),
            level
        )
    }

    fn render_player_rows<'b>(
        &'b self,
        player: &'b PlayerInfo,
        summ_stats_opt: Option<&'b SummonerWithStats>,
    ) -> Vec<Row<'b>> {
        // Player info
        let player_name = if !player.game_name.is_empty() {
            styled_line!("{}#{}", player.game_name, player.tag_line; Color::White)
        } else {
            styled_line!("<Player is private>"; Color::DarkGray)
        };
        let player_cells = vec![
            Cell::from(match player.is_ally {
                Some(true) => styled_line!("Ally"; Color::Blue),
                Some(false) => styled_line!("Enemy"; Color::Red),
                None => styled_line!("N/A"; Color::DarkGray),
            }),
            Cell::from(Self::format_position(&player.position)),
            Cell::from(player_name),
            Cell::from(summ_stats_opt.map_or(styled_span!("?"; Color::DarkGray), |s| {
                s.summoner.level.map_or(styled_span!("---"; Color::DarkGray), |level| {
                    styled_span!(level.to_string())
                })
            })),
        ];

        // Ranked info
        let mut ranked_cells = vec![];
        match summ_stats_opt {
            None => {
                ranked_cells.push(std::iter::repeat_n(Cell::from(styled_span!("?"; Color::DarkGray)), 5).collect_vec())
            }

            Some(summ_stats) => match summ_stats.ranked_stats {
                None => ranked_cells.extend([
                    vec![
                        Cell::from(styled_span!("No data"; Color::DarkGray)),
                        Cell::from(styled_span!("---"; Color::DarkGray)),
                        Cell::from(styled_span!("---"; Color::DarkGray)),
                        Cell::from(styled_span!("---"; Color::DarkGray)),
                    ],
                    std::iter::repeat_n(Cell::from(""), 4).collect::<Vec<_>>(),
                ]),
                Some(ref ranked_stats) => {
                    for queue in &["RANKED_SOLO_5x5", "RANKED_FLEX_SR"] {
                        match ranked_stats.get(*queue) {
                            Some(stats) => {
                                let rank_color = Self::get_rank_color(&stats.tier);
                                ranked_cells.push(vec![
                                    Cell::from(Self::format_queue_type(queue)),
                                    Cell::from(
                                        styled_span!(Self::format_rank(&stats.tier, &stats.division); rank_color),
                                    ),
                                    Cell::from(stats.league_points.to_string()),
                                    Cell::from(styled_line!(
                                        "{:>3}/{:<3} ({:.1} %)",
                                        stats.wins,
                                        stats.losses,
                                        stats.wins as f64 / (stats.wins + stats.losses) as f64 * 100.0
                                    )),
                                ]);
                            }
                            None => ranked_cells.push(vec![
                                Cell::from(Self::format_queue_type(queue)),
                                Cell::from(styled_span!("Unranked"; Color::DarkGray)),
                                Cell::from(""),
                                Cell::from(""),
                            ]),
                        }
                    }
                }
            },
        }

        // Mastery
        let mastery_cells = match &summ_stats_opt.map(|s| &s.champion_mastery) {
            Some(mastery) => match mastery.level_points {
                Some((level, points)) => (
                    Cell::from(Self::format_champion_name(&mastery.champion_name)),
                    Cell::from(styled_span!(Self::format_mastery(level, points); Color::White)),
                ),
                None => (
                    Cell::from(Self::format_champion_name(&mastery.champion_name)),
                    Cell::from(styled_span!("---"; Color::DarkGray)),
                ),
            },
            None => (Cell::from(styled_span!("?"; Color::DarkGray)), Cell::from("")),
        };

        // Final rows
        match ranked_cells.as_slice() {
            [no_data] => {
                let mut all_cells = vec![];
                all_cells.extend(player_cells);
                all_cells.extend(no_data.clone());
                all_cells.push(mastery_cells.0);
                vec![Row::new(all_cells)]
            }
            [soloduo, flex] => {
                let mut all_cells1 = vec![];
                all_cells1.extend(player_cells);
                all_cells1.extend(soloduo.clone());
                all_cells1.push(mastery_cells.0);

                let mut all_cells2 = vec![];
                all_cells2.extend(vec![Cell::from(""), Cell::from(""), Cell::from(""), Cell::from("")]);
                all_cells2.extend(flex.clone());
                all_cells2.push(mastery_cells.1);
                vec![Row::new(all_cells1), Row::new(all_cells2)]
            }
            _ => unreachable!(),
        }
    }
}

impl RenderableView for LivePlayerInfoView {
    fn title(&self) -> &str {
        "Live Game Player Info"
    }

    fn update(&mut self, ctrl: &Controller, _keys: &[crossterm::event::KeyCode]) {
        // Update sources if they are active
        if let Some(cs_data) = &mut self.cs_data {
            cs_data.try_update();
        }
        if let Some(live_data) = &mut self.live_game_data {
            live_data.try_update();
        }
        if let Some(post_data) = &mut self.post_game_data {
            post_data.try_update();
        }
        if let Some(players_data) = &mut self.players_data {
            players_data.try_update();
        }

        // Check if post game finished first (highest priority)
        if let Some(post_data) = &self.post_game_data {
            if !post_data.is_loading() && post_data.error().is_none() {
                if let Some(Some(session)) = post_data.get_data() {
                    let session = session.clone();
                    self.cs_data = None; // Cancel other requests
                    self.live_game_data = None;
                    self.post_game_data = None;

                    // Only update if this is a new post game session
                    let is_new_session = match &self.game_state {
                        Some(GameState::PostGame {
                            session_info: curr_session,
                            ..
                        }) => session != *curr_session,
                        _ => true,
                    };

                    if is_new_session {
                        // PostGame is available, use it
                        let player_team_id = session
                            .teams
                            .iter()
                            .find(|t| t.is_player_team)
                            .and_then(|t| t.players.first())
                            .map(|p| p.team_id);

                        let player_infos = session
                            .teams
                            .iter()
                            .flat_map(|t| &t.players)
                            .map(|p| PlayerInfo {
                                game_name: p.game_name.clone(),
                                tag_line: p.tag_line.clone(),
                                position: p.position.clone(),
                                is_ally: Some(Some(p.team_id) == player_team_id),
                                champion: ctrl.lookup.get_champion_name(&p.champion_name).ok(),
                            })
                            .collect_vec();

                        // Extract player names and fetch ranked info
                        let player_names = player_infos
                            .iter()
                            .map(|p| (p.game_name.clone(), p.tag_line.clone(), p.champion.clone()))
                            .collect_vec();

                        self.game_state = Some(GameState::PostGame {
                            session_info: session,
                            players: player_infos,
                            ranked_info: None,
                        });

                        let rx = ctrl.manager.get_ranked_info(player_names);
                        self.players_data = Some(AsyncData::new(rx));
                    }
                }
            }
        }

        // Check if live game finished
        if let Some(live_data) = &self.live_game_data {
            if !live_data.is_loading() && live_data.error().is_none() {
                if let Some(Some(session)) = live_data.get_data() {
                    let session = session.clone();
                    self.cs_data = None; // Cancel other requests
                    self.live_game_data = None;
                    self.post_game_data = None;

                    // Only update if this is a new live game session
                    let is_new_session = match &self.game_state {
                        Some(GameState::LiveGame {
                            session_info: curr_session,
                            ..
                        }) => session != *curr_session,
                        _ => true,
                    };

                    if is_new_session {
                        // LiveGame is available, use it
                        let ally_team = session
                            .players
                            .iter()
                            .find(|p| (p.game_name.clone(), p.tag_line.clone()) == self.self_info)
                            .map(|p| p.team.clone());
                        let player_infos = session
                            .players
                            .iter()
                            .map(|p| PlayerInfo {
                                game_name: p.game_name.clone(),
                                tag_line: p.tag_line.clone(),
                                position: p.position.clone(),
                                is_ally: ally_team.as_ref().map(|a| p.team == *a),
                                champion: ctrl.lookup.get_champion_name(&p.champion_name).ok(),
                            })
                            .collect_vec();

                        // Extract player names and fetch ranked info
                        let player_names = player_infos
                            .iter()
                            .map(|p| (p.game_name.clone(), p.tag_line.clone(), p.champion.clone()))
                            .collect_vec();

                        self.game_state = Some(GameState::LiveGame {
                            session_info: session,
                            players: player_infos,
                            ranked_info: None,
                        });

                        let rx = ctrl.manager.get_ranked_info(player_names);
                        self.players_data = Some(AsyncData::new(rx));
                    }
                }
            }
        }

        // Check if champ select finished (if we still don't have a game or it's a new session)
        if let Some(cs_data) = &self.cs_data {
            if !cs_data.is_loading() && cs_data.error().is_none() {
                if let Some(Some(session)) = cs_data.get_data() {
                    let session = session.clone();
                    self.cs_data = None;
                    self.live_game_data = None; // Cancel other requests
                    self.post_game_data = None;

                    // Only update if this is a new champ select session
                    let is_new_session = match &self.game_state {
                        Some(GameState::ChampSelect {
                            session_info: curr_session,
                            ..
                        }) => session != *curr_session,
                        _ => true,
                    };

                    if is_new_session {
                        let player_infos = session
                            .my_team
                            .iter()
                            .chain(session.their_team.iter())
                            .map(|p| PlayerInfo {
                                game_name: p.game_name.clone(),
                                tag_line: p.tag_line.clone(),
                                position: p.position.clone(),
                                is_ally: Some(p.is_ally),
                                champion: ctrl.lookup.get_champion(&p.selected_champion).ok(),
                            })
                            .collect_vec();

                        // Extract player names and fetch ranked info
                        let player_names = player_infos
                            .iter()
                            .map(|p| (p.game_name.clone(), p.tag_line.clone(), None))
                            .collect_vec();

                        // New ChampSelect is available, use it
                        self.game_state = Some(GameState::ChampSelect {
                            session_info: session,
                            players: player_infos,
                            ranked_info: None,
                        });

                        let rx = ctrl.manager.get_ranked_info(player_names);
                        self.players_data = Some(AsyncData::new(rx));
                    }
                }
            }
        }

        // If ranked player data finished, merge it into the game state
        if let Some(players_data) = &self.players_data {
            if !players_data.is_loading() {
                if let Some(players) = players_data.get_data() {
                    if let Some(gs) = &mut self.game_state {
                        let cloned = players.clone();
                        match gs {
                            GameState::ChampSelect {
                                ranked_info: players, ..
                            } => *players = Some(cloned),
                            GameState::LiveGame {
                                ranked_info: players, ..
                            } => *players = Some(cloned),
                            GameState::PostGame {
                                ranked_info: players, ..
                            } => *players = Some(cloned),
                            _ => {}
                        }
                    }
                    self.players_data = None;
                } else if let Some(err) = players_data.error() {
                    // TODO ?
                    self.game_state = Some(GameState::Error(err.to_string()));
                    self.players_data = None;
                }
            }
        }

        // If all session requests finished but we still have no game state, decide NotInGame vs Error
        let cs_finished = self.cs_data.as_ref().is_some_and(|d| !d.is_loading());
        let live_finished = self.live_game_data.as_ref().is_some_and(|d| !d.is_loading());
        let post_finished = self.post_game_data.as_ref().is_some_and(|d| !d.is_loading());
        if cs_finished && live_finished && post_finished {
            self.cs_data = None;
            self.live_game_data = None;
            self.post_game_data = None;
            self.players_data = None;

            let mut error = String::new();
            if let Some(cs_err) = self.cs_data.as_ref().and_then(|d| d.error()) {
                error.push_str(&format!("Champ Select Error: {}\n", cs_err));
            }
            if let Some(live_err) = self.live_game_data.as_ref().and_then(|d| d.error()) {
                error.push_str(&format!("Live Game Error: {}\n", live_err));
            }
            if let Some(post_err) = self.post_game_data.as_ref().and_then(|d| d.error()) {
                error.push_str(&format!("Post Game Error: {}\n", post_err));
            }

            if error.is_empty() {
                self.game_state = Some(GameState::NotInGame);
            } else {
                self.game_state = Some(GameState::Error(error));
            }
        }
    }

    fn auto_refresh_interval(&self) -> Option<f32> {
        match self.game_state {
            None => Some(1.0),
            Some(GameState::Error(_)) => None,
            Some(GameState::NotInGame) => Some(1.0),
            Some(_) => Some(10.0),
        }
    }

    fn refresh_data(&mut self, controller: &Controller) -> Result<(), String> {
        // Only refresh if we're not currently loading
        let is_loading = self.cs_data.as_ref().is_some_and(|d| d.is_loading())
            || self.live_game_data.as_ref().is_some_and(|d| d.is_loading())
            || self.post_game_data.as_ref().is_some_and(|d| d.is_loading())
            || self.players_data.as_ref().is_some_and(|d| d.is_loading());

        if !is_loading {
            self.start_session_requests(controller);
        }
        Ok(())
    }

    fn render(&self, rc: RenderContext) -> ViewResult {
        // Decide what to render based on the game state
        let Some(game_state) = &self.game_state else {
            let loading_text = vec![styled_line!("Loading game data...")];
            let paragraph = ratatui::widgets::Paragraph::new(loading_text)
                .block(rc.block)
                .wrap(ratatui::widgets::Wrap { trim: false });
            rc.frame.render_widget(paragraph, rc.area);
            return Ok(());
        };

        match game_state {
            GameState::NotInGame => {
                let not_in_game_text = vec![
                    styled_line!(),
                    styled_line!("  Not in a champ select / live game / post game. Waiting for game to start..."; Color::Red),
                ];
                let paragraph = ratatui::widgets::Paragraph::new(not_in_game_text)
                    .block(rc.block)
                    .wrap(ratatui::widgets::Wrap { trim: false });
                rc.frame.render_widget(paragraph, rc.area);
                Ok(())
            }
            GameState::Error(msg) => {
                rc.error(msg);
                Ok(())
            }
            GameState::ChampSelect {
                players, ranked_info, ..
            }
            | GameState::LiveGame {
                players, ranked_info, ..
            }
            | GameState::PostGame {
                players, ranked_info, ..
            } => {
                let summoners = ranked_info.as_ref();

                // Render rows
                let mut ally_rows = vec![];
                let mut enemy_rows = vec![];

                for player in players.iter().sorted_by_key(|p| {
                    (
                        !p.is_ally.unwrap_or(false),
                        Self::position_sort_key(&p.position),
                        p.game_name.clone(),
                        p.tag_line.clone(),
                    )
                }) {
                    let stats = summoners.and_then(|ss| {
                        ss.iter().find(|s| {
                            s.summoner.game_name == player.game_name && s.summoner.tag_line == player.tag_line
                        })
                    });

                    let target_vec = if player.is_ally.unwrap_or(true) {
                        &mut ally_rows
                    } else {
                        &mut enemy_rows
                    };
                    target_vec.extend(self.render_player_rows(player, stats));
                    target_vec.push(empty_row!(9));
                }

                // Combine and add separator if enemies are given
                let mut rows = vec![];
                rows.extend(ally_rows);
                if !enemy_rows.is_empty() {
                    rows.push(empty_row!(9));
                    rows.push(empty_row!(9));
                    rows.extend(enemy_rows);
                }

                let extra_title = self.game_state.as_ref().map_or("".to_string(), |gs| match gs {
                    GameState::ChampSelect { .. } => "(Champ Select)".to_string(),
                    GameState::LiveGame { .. } => "(Live Game)".to_string(),
                    GameState::PostGame { .. } => "(Post Game)".to_string(),
                    _ => "".to_string(),
                });

                // Render table
                let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

                let table = Table::new(visible_rows, self.columns())
                    .header(
                        header_row!("Team", "Position", "Player", "Level", "Queue", "Rank", "LP", "W/L", "Mastery")
                            .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                            .bottom_margin(1),
                    )
                    .block(rc.block.title(extra_title))
                    .column_spacing(2)
                    .style(Style::default().fg(Color::White));

                // Reserve the table area and render it
                rc.frame.render_widget(table, rc.area);

                // Add hint text below the table (same horizontal area, one line from bottom)
                let hint = styled_line!(
                    "Note: Ranked / Mastery info may take up to a minute on first request."; Color::DarkGray
                )
                .alignment(Alignment::Center);
                let hint_paragraph = Paragraph::new(vec![hint]).block(Block::default());

                // Place hint at the bottom line of the area if possible (respect the block of table)
                let mut hint_area = rc.area;
                if hint_area.height > 0 {
                    hint_area.x += 1;
                    hint_area.width = hint_area.width.saturating_sub(2);
                    hint_area.y += hint_area.height.saturating_sub(2);
                    hint_area.height = 1;
                    rc.frame.render_widget(hint_paragraph, hint_area);
                }

                Ok(())
            }
        }
    }
}
