use crate::{
    impl_text_view, styled_text,
    ui::{Controller, TextCreationResult},
};

// ============================================================================
// Summoner Info View
// ============================================================================

fn summoner_info_view(ctrl: &Controller) -> TextCreationResult {
    let summoner = ctrl.manager.get_summoner();
    let lines = vec![
        styled_text!(),
        styled_text!("Game Name:      {}", summoner.game_name),
        styled_text!("Tag Line:       {}", summoner.tag_line),
        styled_text!("Level:          {}", summoner.level),
        styled_text!(),
        styled_text!(Color::DarkGray, "ID:             {}", summoner.id),
        styled_text!(Color::DarkGray, "PUUID:          {}", summoner.puuid),
    ];
    Ok(lines)
}

impl_text_view!(
    SummonerInfoView,
    summoner_info_view,
    "Show Summoner Info",
    "Show Summoner Info"
);
