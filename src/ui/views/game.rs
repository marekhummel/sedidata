use ratatui::style::Color;

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
                if mastery.level > 5 {
                    output.push_str(&format!(" ({}/{} tokens, {} pts)", mastery.marks, 2, mastery.points))
                } else {
                    output.push_str(&format!(" ({} pts)", mastery.points));
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
            lines.push(styled_line!("Currently selected champ:"; Color::Cyan));
            let current_champ = champ_select_info.current_champ_id;
            lines.push(styled_line!(
                "{}",
                format_selectable_champ(ctrl.lookup, &current_champ)?
            ));

            lines.push(styled_line!());
            lines.push(styled_line!("Benched Champions:"; Color::Cyan));
            for bench_champ in champ_select_info.benched_champs {
                lines.push(styled_line!("{}", format_selectable_champ(ctrl.lookup, &bench_champ)?));
            }

            lines.push(styled_line!());
            lines.push(styled_line!("Tradable Champions:"; Color::Cyan));
            for team_champ in champ_select_info.team_champs {
                lines.push(styled_line!("{}", format_selectable_champ(ctrl.lookup, &team_champ)?));
            }
        }
        None => lines.extend(vec![styled_line!(), styled_line!("  Not in champ select!"; Color::Red)]),
    };
    Ok(lines)
}

impl_text_view!(ChampSelectInfoView, champ_select_info_view, "Champ Select Info");
