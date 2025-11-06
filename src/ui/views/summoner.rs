use crate::{
    impl_text_view,
    ui::{Controller, TextCreationResult},
};

// ============================================================================
// Summoner Info View
// ============================================================================

fn summoner_info_view(ctrl: &Controller) -> TextCreationResult {
    let summoner = ctrl.manager.get_summoner();
    let lines = [
        String::new(),
        format!("Game Name:  {}", summoner.game_name),
        format!("Tag Line:   {}", summoner.tag_line),
        format!("Level:      {}", summoner.level),
        String::new(),
        format!("ID:         {}", summoner.id),
        format!("PUUID:      {}", summoner.puuid),
    ];
    Ok(lines.join("\n"))
}

impl_text_view!(
    SummonerInfoView,
    summoner_info_view,
    "Show Summoner Info",
    "Show Summoner Info"
);
