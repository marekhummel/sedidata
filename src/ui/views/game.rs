use ratatui::style::Color;

use crate::{
    impl_text_view,
    model::{champion::Champion, ids::ChampionId, mastery::Mastery},
    service::lookup::LookupService,
    styled_line,
    ui::{Controller, TextCreationResult, ViewError},
};

// ============================================================================
// Champion Select Info View
// ============================================================================

type ChampionSelectEntry = (Champion, Option<Mastery>);

fn get_champ_info(champ: &ChampionId, lookup: &LookupService) -> Result<ChampionSelectEntry, ViewError> {
    let champion = lookup.get_champion(champ)?;
    let mastery = match champion.owned {
        true => lookup.get_mastery(champ).cloned().ok(),
        false => None,
    };

    Ok((champion.clone(), mastery))
}

fn format_selectable_champ(entry: ChampionSelectEntry) -> Result<String, ViewError> {
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

fn get_entries(champ_ids: &[ChampionId], lookup: &LookupService) -> Result<Vec<ChampionSelectEntry>, ViewError> {
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

fn champ_select_info_view(ctrl: &Controller) -> TextCreationResult {
    // Verify queue id

    let mut lines = Vec::new();

    match ctrl.manager.get_champ_select_info()? {
        Some(champ_select_info) => {
            lines.push(styled_line!("Currently selected champ:"; Color::Rgb(200, 150, 0)));
            let current_champ = champ_select_info.current_champ_id;
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

            let team = get_entries(&champ_select_info.team_champs, ctrl.lookup)?;
            for entry in team {
                lines.push(styled_line!("{}", format_selectable_champ(entry)?));
            }
        }
        None => lines.extend(vec![styled_line!(), styled_line!("  Not in champ select!"; Color::Red)]),
    };
    Ok(lines)
}

impl_text_view!(ChampSelectInfoView, champ_select_info_view, "Champ Select Info", auto_refresh: 0.5);
