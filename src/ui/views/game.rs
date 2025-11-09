use crate::{
    impl_text_view,
    model::ids::ChampionId,
    service::lookup::LookupService,
    styled_line,
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
    Ok(output)
}

fn champ_select_info_view(ctrl: &Controller) -> TextCreationResult {
    let mut lines = Vec::new();

    match ctrl.manager.get_champ_select_info()? {
        Some(champ_select_info) => {
            lines.push(styled_line!("Currently selected champ:"; Cyan));
            let current_champ = champ_select_info.current_champ_id;
            lines.push(styled_line!(
                "{}",
                format_selectable_champ(ctrl.lookup, &current_champ)?
            ));

            lines.push(styled_line!());
            lines.push(styled_line!("Benched Champions:"; Cyan));
            for bench_champ in champ_select_info.benched_champs {
                lines.push(styled_line!("{}", format_selectable_champ(ctrl.lookup, &bench_champ)?));
            }

            lines.push(styled_line!());
            lines.push(styled_line!("Tradable Champions:"; Cyan));
            for team_champ in champ_select_info.team_champs {
                lines.push(styled_line!("{}", format_selectable_champ(ctrl.lookup, &team_champ)?));
            }
        }
        None => lines.push(styled_line!("Not in champ select!"; Red)),
    };
    Ok(lines)
}

impl_text_view!(
    ChampSelectInfoView,
    champ_select_info_view,
    "Champ Select Info",
    "Champ Select Info"
);
