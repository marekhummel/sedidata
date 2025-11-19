use std::vec;

use crate::{
    empty_row, header_row, impl_text_view,
    model::{
        champion::Champion,
        champselect::{ChampSelectPlayerInfo, ChampSelectSession},
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
use itertools::{EitherOrBoth, Itertools};
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table},
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

            match queue.select_mode_group.as_str() {
                "kARAM" => {
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
    cs_data: AsyncData<Option<ChampSelectSession>>,
    players_data: Option<AsyncData<Vec<SummonerWithStats>>>,
}

impl LivePlayerInfoView {
    pub fn new(ctrl: &Controller) -> Self {
        // Execute the first fetch immediately (champ select)
        let rx = ctrl.manager.get_champ_select();

        Self {
            cs_data: AsyncData::new(rx),
            players_data: None,
        }
    }

    fn columns(&self) -> [Constraint; 8] {
        [
            Constraint::Length(6),  // Team
            Constraint::Length(12), // Position
            Constraint::Length(30), // Player Name
            Constraint::Length(6),  // Level
            Constraint::Length(14), // Queue Type
            Constraint::Length(12), // Rank
            Constraint::Length(5),  // LP
            Constraint::Length(20), // Wins / Losses
        ]
    }

    fn format_position(position: &str) -> &str {
        match position {
            "top" => "Top",
            "jungle" => "Jungle",
            "middle" => "Mid",
            "bottom" => "Bot",
            "utility" => "Support",
            _ => "Fill",
        }
    }

    fn position_sort_key(position: &str) -> u8 {
        match position {
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

    fn render_player_rows<'b>(
        &'b self,
        player: &'b ChampSelectPlayerInfo,
        summ_stats_opt: Option<&'b SummonerWithStats>,
    ) -> Vec<Row<'b>> {
        // Player info
        let player_name = format!("{}#{}", player.game_name, player.tag_line);
        let player_cells = vec![
            Cell::from(if player.is_ally {
                styled_line!("Ally"; Color::Blue)
            } else {
                styled_line!("Enemy"; Color::Red)
            }),
            Cell::from(Self::format_position(&player.position)),
            Cell::from(player_name),
            Cell::from(summ_stats_opt.map_or("?".to_string(), |s| {
                s.summoner.level.map_or("---".to_string(), |level| level.to_string())
            })),
        ];

        // Ranked info
        let mut ranked_cells = vec![];
        match summ_stats_opt {
            None => ranked_cells.push(vec![
                Cell::from(styled_span!("?"; Color::DarkGray)),
                Cell::from(styled_span!("?"; Color::DarkGray)),
                Cell::from(styled_span!("?"; Color::DarkGray)),
                Cell::from(styled_span!("?"; Color::DarkGray)),
            ]),
            Some(summ_stats) => match summ_stats.ranked_stats {
                None => ranked_cells.push(vec![
                    Cell::from(styled_span!("No data"; Color::DarkGray)),
                    Cell::from(styled_span!("---"; Color::DarkGray)),
                    Cell::from(styled_span!("---"; Color::DarkGray)),
                    Cell::from(styled_span!("---"; Color::DarkGray)),
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
                                Cell::from("Unranked"),
                                Cell::from(""),
                                Cell::from(""),
                            ]),
                        }
                    }

                    for (queue, stats) in ranked_stats
                        .iter()
                        .filter(|q| q.0 != "RANKED_SOLO_5x5" && q.0 != "RANKED_FLEX_SR")
                    {
                        let rank_color = Self::get_rank_color(&stats.tier);
                        ranked_cells.push(vec![
                            Cell::from(Self::format_queue_type(queue)),
                            Cell::from(styled_span!(Self::format_rank(&stats.tier, &stats.division); rank_color)),
                            Cell::from(stats.league_points.to_string()),
                            Cell::from(styled_line!(
                                "{:>3}/{:>3} ({:.1} %)",
                                stats.wins,
                                stats.losses,
                                stats.wins as f64 / (stats.wins + stats.losses) as f64 * 100.0
                            )),
                        ]);
                    }
                }
            },
        }

        ranked_cells
            .into_iter()
            .zip_longest(vec![player_cells])
            .map(|zip| match zip {
                EitherOrBoth::Both(ranked, player) => {
                    let mut all_cells = vec![];
                    all_cells.extend(player);
                    all_cells.extend(ranked);
                    Row::new(all_cells)
                }
                EitherOrBoth::Left(ranked) => {
                    let mut all_cells = vec![Cell::from(""), Cell::from(""), Cell::from(""), Cell::from("")];
                    all_cells.extend(ranked);
                    Row::new(all_cells)
                }
                EitherOrBoth::Right(_) => unreachable!(),
            })
            .collect::<Vec<_>>()
    }
}

impl RenderableView for LivePlayerInfoView {
    fn title(&self) -> &str {
        "Live Game Player Info"
    }

    fn update(&mut self, ctrl: &Controller, _keys: &[crossterm::event::KeyCode]) {
        // Update champ select data
        self.cs_data.try_update();

        // If champ select is loaded and we haven't started the players fetch yet, extract player names
        if !self.cs_data.is_loading() && self.players_data.is_none() {
            if let Some(Some(session)) = self.cs_data.get_data() {
                // Extract player names for later use in refresh
                let player_names: Vec<_> = session
                    .my_team
                    .iter()
                    .chain(session.their_team.iter())
                    .map(|pi| (pi.game_name.clone(), pi.tag_line.clone()))
                    .collect();

                let rx = ctrl.manager.get_ranked_info(player_names);
                self.players_data = Some(AsyncData::new(rx));
            }
        }

        // Update players data if it exists
        if let Some(players_data) = &mut self.players_data {
            players_data.try_update();
        }
    }

    fn refresh_data(&mut self, controller: &Controller) -> Result<(), String> {
        let rx = controller.manager.get_champ_select();

        self.cs_data = AsyncData::new(rx);
        self.players_data = None;
        Ok(())
    }

    fn render(&self, rc: RenderContext) -> ViewResult {
        // Check if champ select is still loading
        if self.cs_data.is_loading() {
            let loading_text = vec![styled_line!("Loading champ select data...")];
            let paragraph = ratatui::widgets::Paragraph::new(loading_text)
                .block(rc.block)
                .wrap(ratatui::widgets::Wrap { trim: false });
            rc.frame.render_widget(paragraph, rc.area);
            return Ok(());
        }

        // Check for error in champ select fetch
        if let Some(error) = self.cs_data.error() {
            rc.error(error);
            return Ok(());
        }

        // Get the champ select data
        let cs_session = match self.cs_data.get_data() {
            Some(Some(session)) => session,
            Some(None) => {
                rc.error("Not in champ select!");
                return Ok(());
            }
            None => {
                rc.error("Champ select data not available");
                return Ok(());
            }
        };

        // Check if player data request finsihed with an error
        let players_data = self.players_data.as_ref().unwrap();
        if let Some(error) = players_data.error() {
            rc.error(error);
            return Ok(());
        }

        let mut rows = vec![];

        // Sort players by team and position
        let mut sorted_cs = cs_session
            .my_team
            .iter()
            .chain(cs_session.their_team.iter())
            .collect_vec();
        sorted_cs.sort_by_key(|p| (p.is_ally, Self::position_sort_key(&p.position)));

        let summoners = players_data.get_data();

        for player in &sorted_cs {
            let stats = summoners.map(|ss| {
                ss.iter()
                    .find(|s| s.summoner.game_name == player.game_name && s.summoner.tag_line == player.tag_line)
                    .unwrap()
            });

            rows.extend(self.render_player_rows(player, stats));
            rows.push(empty_row!(8));
        }

        // Skip rows based on scroll offset
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, self.columns())
            .header(
                header_row!("Team", "Position", "Player", "Level", "Queue", "Rank", "LP", "W/L")
                    .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                    .bottom_margin(1),
            )
            .block(rc.block)
            .column_spacing(2)
            .style(Style::default().fg(Color::White));
        rc.frame.render_widget(table, rc.area);

        Ok(())
    }
}
