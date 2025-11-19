use ratatui::style::Color;

use crate::{
    impl_text_view, styled_line,
    ui::{Controller, TextCreationResult},
};

// ============================================================================
// Summoner Info View
// ============================================================================

fn summoner_info_view(ctrl: &Controller) -> TextCreationResult {
    let summoner = ctrl.manager.get_summoner();
    let lines = vec![
        styled_line!(),
        styled_line!("Game Name:      {}", summoner.game_name),
        styled_line!("Tag Line:       {}", summoner.tag_line),
        styled_line!(
            "Level:          {}",
            summoner.level.map_or("-".to_string(), |l| l.to_string())
        ),
        styled_line!(),
        styled_line!("ID:             {}",summoner.id; Color::DarkGray),
        styled_line!("PUUID:          {}", summoner.puuid; Color::DarkGray),
    ];
    Ok(lines)
}

impl_text_view!(SummonerInfoView, summoner_info_view, "Show Summoner Info");
