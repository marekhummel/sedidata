use std::vec;

use crate::{
    empty_row, header_row, impl_text_view,
    model::{
        champion::Champion,
        champselect::{ChampSelectPlayer, ChampSelectSession},
        ids::ChampionId,
        mastery::Mastery,
    },
    service::lookup::LookupService,
    styled_line, styled_span,
    ui::{views::RenderableView, Controller, RenderContext, TextCreationResult, ViewError, ViewResult},
};
use itertools::{EitherOrBoth, Itertools};
use ratatui::{
    layout::{Alignment, Constraint},
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
        true => lookup.get_mastery(champ).cloned().ok(),
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

    match ctrl.manager.get_champ_select()? {
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
    _cs_session: Option<ChampSelectSession>,
    players: Vec<ChampSelectPlayer>,
    error: Option<String>,
}

impl LivePlayerInfoView {
    pub fn new(ctrl: &Controller) -> Self {
        match ctrl.manager.get_champ_select_with_ranked() {
            Ok(champ_select) => match champ_select {
                Some((session, players)) => Self {
                    _cs_session: Some(session),
                    players,
                    error: None,
                },
                None => Self {
                    _cs_session: None,
                    players: Vec::new(),
                    error: Some("  Not in champ select!".into()),
                },
            },
            Err(e) => Self {
                _cs_session: None,
                players: Vec::new(),
                error: Some(format!("Failed to load player data: {}", e)),
            },
        }
    }

    fn columns(&self) -> [Constraint; 9] {
        [
            Constraint::Length(6),  // Team
            Constraint::Length(12), // Position
            Constraint::Length(30), // Player Name
            Constraint::Length(6),  // Level
            Constraint::Length(14), // Queue Type
            Constraint::Length(12), // Rank
            Constraint::Length(5),  // LP
            Constraint::Length(8),  // Wins
            Constraint::Length(18), // Peak Rank
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

    fn queue_sort_key(queue: &str) -> u8 {
        match queue {
            "RANKED_SOLO_5x5" => 0,
            "RANKED_FLEX_SR" => 1,
            _ => 2,
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

    fn render_player_rows<'a>(&'a self, player: &'a ChampSelectPlayer) -> Vec<Row<'a>> {
        // Player info
        let summoner = player.summoner.as_ref().unwrap();
        let player_name = format!("{}#{}", summoner.game_name, summoner.tag_line);
        let player_cells = vec![
            Cell::from(if player.player_info.is_ally {
                styled_line!("Ally"; Color::Blue)
            } else {
                styled_line!("Enemy"; Color::Red)
            }),
            Cell::from(Self::format_position(&player.player_info.position)),
            Cell::from(player_name),
            Cell::from(summoner.level.to_string()),
        ];

        // Ranked info
        let mut ranked_cells = vec![];
        if player.ranked_stats.is_empty() {
            ranked_cells.push(vec![
                Cell::from("No ranked data"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]);
        } else {
            for stats in &player.ranked_stats {
                let rank_color = Self::get_rank_color(&stats.tier);
                let peak_rank_color = Self::get_rank_color(&stats.highest_tier);
                ranked_cells.push(vec![
                    Cell::from(Self::format_queue_type(&stats.queue_type)),
                    Cell::from(styled_span!(Self::format_rank(&stats.tier, &stats.division); rank_color)),
                    Cell::from(styled_line!(stats.league_points).alignment(Alignment::Right)),
                    Cell::from(styled_line!(stats.wins).alignment(Alignment::Right)),
                    Cell::from(styled_line!(
                        Self::format_rank(&stats.highest_tier, &stats.highest_division); peak_rank_color)),
                ]);
            }
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

    fn render(&self, rc: RenderContext) -> ViewResult {
        if let Some(error) = &self.error {
            rc.error(error);
            return Ok(());
        }

        let mut rows = vec![];

        // Sort players by team and position, and sort their ranked stats
        let mut sorted_players = self.players.clone();
        sorted_players.sort_by_key(|p| (p.player_info.is_ally, Self::position_sort_key(&p.player_info.position)));

        // Sort ranked stats for each player
        for player in &mut sorted_players {
            player.ranked_stats.sort_by_key(|s| Self::queue_sort_key(&s.queue_type));
        }

        for player in &sorted_players {
            rows.extend(self.render_player_rows(player));
            rows.push(empty_row!(9));
        }

        // Skip rows based on scroll offset
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, self.columns())
            .header(
                header_row!(
                    "Team",
                    "Position",
                    "Player",
                    "Level",
                    "Queue",
                    "Rank",
                    "LP",
                    "Wins",
                    "Peak Rank"
                )
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
