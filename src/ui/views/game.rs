use crate::{
    impl_text_view,
    model::ids::ChampionId,
    service::lookup::LookupService,
    ui::{Controller, TextCreationResult, ViewError},
};

// ============================================================================
// Champion Select Info View
// ============================================================================

fn format_selectable_champ(lookup: &LookupService, champ: &ChampionId) -> Result<String, ViewError> {
    let champion = lookup.get_champion(champ)?;
    let mut output = format!("  {:<16}", format!("{}:", champion.name));
    match champion.owned {
        true => match lookup.get_mastery(champ) {
            Ok(mastery) => {
                output.push_str(&format!("  Level {}", mastery.level));
                match mastery.tokens {
                    Some(tokens) => output.push_str(&format!(
                        " ({}/{} tokens, {} pts)",
                        tokens,
                        mastery.level - 3,
                        mastery.points
                    )),
                    None => output.push_str(&format!(" ({} pts)", mastery.points)),
                }
            }
            Err(_) => output.push_str("  Level 0 (not played!)"),
        },
        false => output.push_str("  not owned!"),
    }
    output.push('\n');
    Ok(output)
}

fn champ_select_info_view(ctrl: &Controller) -> TextCreationResult {
    let mut result = String::new();

    match ctrl.manager.get_champ_select_info()? {
        Some(champ_select_info) => {
            result.push_str("Currently selected champ:\n");
            let current_champ = champ_select_info.current_champ_id;
            result.push_str(&format_selectable_champ(ctrl.lookup, &current_champ)?);

            result.push_str("\nBenched Champions:\n");
            for bench_champ in champ_select_info.benched_champs {
                result.push_str(&format_selectable_champ(ctrl.lookup, &bench_champ)?);
            }

            result.push_str("\nTradable Champions:\n");
            for team_champ in champ_select_info.team_champs {
                result.push_str(&format_selectable_champ(ctrl.lookup, &team_champ)?);
            }
        }
        None => result.push_str("Not in champ select!"),
    };
    Ok(result)
}

impl_text_view!(
    ChampSelectInfoView,
    champ_select_info_view,
    "Champ Select Info",
    "Champ Select Info"
);
