use ratatui::style::Color;

use crate::{
    impl_text_view, styled_line, styled_span,
    ui::{Controller, TextCreationResult},
};
use std::collections::HashSet;

// ============================================================================
// Champions Without Skin View
// ============================================================================

fn champions_without_skin_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.util.get_owned_champions().recv().unwrap()?;
    let skins = ctrl.util.get_owned_nobase_skins().recv().unwrap()?;
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
    lines.push(styled_line!("{} champion(s) total", champs_no_skin.len(); Color::Rgb(200, 150, 0)));

    Ok(lines)
}

impl_text_view!(
    ChampionsWithoutSkinView,
    champions_without_skin_view,
    "Champions Without Skin"
);

// ============================================================================
// Chromas Without Skin View
// ============================================================================

fn chromas_without_skin_view(ctrl: &Controller) -> TextCreationResult {
    let skins = ctrl.util.get_owned_skins_set().recv().unwrap()?;
    let chromas = ctrl.util.get_owned_chromas().recv().unwrap()?;
    let chromas_no_skin = chromas
        .iter()
        .filter(|ch| !skins.contains(&ch.skin_id))
        .collect::<Vec<_>>();

    let skin_shards = &ctrl.manager.get_loot().recv().unwrap()?.skin_shards;
    let available_skin_shards = chromas_no_skin
        .iter()
        .filter(|c| skin_shards.iter().any(|ss| ss.skin_id == c.skin_id))
        .map(|c| c.skin_id.clone())
        .collect::<Vec<_>>();

    let mut lines = vec![
        styled_line!("Owned chromas for which the skin isn't owned:"),
        styled_line!(),
    ];

    for chroma in chromas_no_skin {
        let skin = ctrl.lookup.get_skin(&chroma.skin_id)?;
        let champ = ctrl.lookup.get_champion(&skin.champ_id)?;
        let chroma_str = format!("{} ({}):", skin.name, champ.name);

        let shard_owned = available_skin_shards.contains(&chroma.skin_id);
        let shard_status = if shard_owned {
            "SKIN SHARD OWNED"
        } else {
            "no skin shard either"
        };
        let shard_color = if shard_owned { Color::Green } else { Color::DarkGray };
        lines.push(styled_line!(LIST [
            styled_span!("  • {:<30} {}   ", chroma_str, chroma.id),
            styled_span!("- {}", shard_status; shard_color)
        ]));
    }

    Ok(lines)
}

impl_text_view!(
    ChromasWithoutSkinView,
    chromas_without_skin_view,
    "Chromas Without Skin"
);
