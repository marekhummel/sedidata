use crate::{
    impl_text_view, styled_line,
    ui::{Controller, TextCreationResult},
};
use std::collections::HashSet;

// ============================================================================
// Champions Without Skin View
// ============================================================================

fn champions_without_skin_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.util.get_owned_champions()?;
    let skins = ctrl.util.get_owned_nobase_skins()?;
    let champs_with_skin = skins.iter().map(|s| s.champ_id.clone()).collect::<HashSet<_>>();
    let mut champs_no_skin = champs
        .iter()
        .filter(|c| !champs_with_skin.contains(&c.id))
        .collect::<Vec<_>>();
    champs_no_skin.sort_by_key(|c| c.name.clone());

    let mut lines = vec![
        styled_line!("Owned champions for which no skin is owned:"),
        styled_line!(),
    ];

    for champ in &champs_no_skin {
        lines.push(styled_line!("  • {}", champ.name));
    }

    lines.push(styled_line!());
    lines.push(styled_line!("{} champion(s) total", champs_no_skin.len(); Cyan));

    Ok(lines)
}

impl_text_view!(
    ChampionsWithoutSkinView,
    champions_without_skin_view,
    "Champions Without Skin",
    "Champions Without Skin"
);

// ============================================================================
// Chromas Without Skin View
// ============================================================================

fn chromas_without_skin_view(ctrl: &Controller) -> TextCreationResult {
    let skins = ctrl.util.get_owned_skins_set()?;
    let chromas = ctrl.util.get_owned_chromas()?;
    let chromas_no_skin = chromas.iter().filter(|ch| !skins.contains(&ch.skin_id));

    let mut lines = vec![
        styled_line!("Owned chromas for which the skin isn't owned:"),
        styled_line!(),
    ];

    for chroma in chromas_no_skin {
        let skin = ctrl.lookup.get_skin(&chroma.skin_id)?;
        let champ = ctrl.lookup.get_champion(&skin.champ_id)?;
        let chroma_str = format!("{} ({}):", skin.name, champ.name);
        lines.push(styled_line!("  • {:<30} {}", chroma_str, chroma.id));
    }

    Ok(lines)
}

impl_text_view!(
    ChromasWithoutSkinView,
    chromas_without_skin_view,
    "Chromas Without Skin",
    "Chromas Without Skin"
);
