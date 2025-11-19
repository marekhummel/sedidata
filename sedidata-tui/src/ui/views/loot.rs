use ratatui::style::Color;

use crate::{
    impl_text_view, styled_line, styled_span,
    ui::{Controller, TextCreationResult},
};
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

// ============================================================================
// Blue Essence Overview View
// ============================================================================

fn blue_essence_overview_view(ctrl: &Controller) -> TextCreationResult {
    let loot = ctrl.manager.get_loot().recv().unwrap()?;
    let be = loot.credits.blue_essence;
    let champ_shards = &loot.champion_shards;

    let convertable = champ_shards
        .iter()
        .map(|cs| cs.count as u32 * cs.disenchant_value as u32)
        .sum::<u32>();

    let keep1 = champ_shards
        .iter()
        .map(|cs| max(cs.count as i8 - 1, 0) as u32 * cs.disenchant_value as u32)
        .sum::<u32>();

    let keep2 = champ_shards
        .iter()
        .map(|cs| max(cs.count as i8 - 2, 0) as u32 * cs.disenchant_value as u32)
        .sum::<u32>();

    let lines = vec![
        styled_line!(),
        styled_line!(LIST [
            styled_span!("Current BE:                                 "),
            styled_span!(be; Bold Color::Rgb(200, 150, 0)),
        ]),
        styled_line!("Convertable BE:                             {}", convertable),
        styled_line!("Convertable BE (Keep one shard per champ):  {}", keep1),
        styled_line!("Convertable BE (Keep two shards per champ): {}", keep2),
    ];
    Ok(lines)
}

impl_text_view!(BlueEssenceOverviewView, blue_essence_overview_view, "Blue Essence Info");

// ============================================================================
// Missing Champion Shards View
// ============================================================================

fn missing_champ_shards_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.manager.get_champions().recv().unwrap()?;
    let loot = ctrl.manager.get_loot().recv().unwrap()?;
    let owned_champ_shards = loot
        .champion_shards
        .iter()
        .map(|cs| cs.champ_id.clone())
        .collect::<HashSet<_>>();

    let mut missing_cs = champs
        .iter()
        .filter(|c| !owned_champ_shards.contains(&c.id))
        .collect::<Vec<_>>();
    missing_cs.sort_by_key(|c| c.name.as_str());

    let mut lines = vec![
        styled_line!("Champions for which no champ shard is owned:"),
        styled_line!(),
    ];

    for c in &missing_cs {
        lines.push(styled_line!("  • {}", c.name));
    }

    lines.push(styled_line!());
    lines.push(styled_line!("{} champ(s) total", missing_cs.len(); Color::Rgb(200, 150, 0)));
    Ok(lines)
}

impl_text_view!(
    MissingChampShardsView,
    missing_champ_shards_view,
    "Missing Champion Shards"
);

// ============================================================================
// Interesting Skins View
// ============================================================================

fn interesting_skins_view(ctrl: &Controller) -> TextCreationResult {
    let sorted_champs = ctrl
        .util
        .get_champions_sorted_by_mastery(None, Some(10_000))
        .recv()
        .unwrap()?;
    let skin_shards = &ctrl.manager.get_loot().recv().unwrap()?.skin_shards;

    let mut lines = vec![
        styled_line!("Owned skin shards for champs with 10k or more mastery points (sorted by mastery points):"),
        styled_line!(),
    ];

    for c in sorted_champs {
        let shards = skin_shards
            .iter()
            .filter(|ss| ctrl.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c);

        let champ_name = ctrl.lookup.get_champion(&c)?.name.to_string();
        let mut first = true;
        for shard in shards {
            let skin = ctrl.lookup.get_skin(&shard.skin_id)?;
            if first {
                lines.push(styled_line!(LIST [
                    styled_span!(format!("{:<16}  ", format!("{}:", champ_name)); Bold Color::White),
                    styled_span!(skin.name),
                ]));
                first = false;
            } else {
                lines.push(styled_line!("{:<16}  {}", "", skin.name));
            }
        }
    }
    Ok(lines)
}

impl_text_view!(InterestingSkinsView, interesting_skins_view, "Interesting Skins");

// ============================================================================
// Skin Shards for First Skin View
// ============================================================================

fn skin_shards_first_skin_view(ctrl: &Controller) -> TextCreationResult {
    let skin_shards = &ctrl.manager.get_loot().recv().unwrap()?.skin_shards;
    let skins = ctrl.util.get_owned_nobase_skins().recv().unwrap()?;
    let champs_with_skin = skins.iter().map(|s| s.champ_id.clone()).collect::<HashSet<_>>();

    let sorted_champs = ctrl.util.get_champions_sorted_by_mastery(None, None).recv().unwrap()?;
    let champs_no_skin = sorted_champs.into_iter().filter(|cid| !champs_with_skin.contains(cid));

    let mut lines = vec![
        styled_line!("Shows skin shards which would be the first skin for the champ (sorted by mastery points):"),
        styled_line!(),
    ];

    for c in champs_no_skin {
        let shards = skin_shards
            .iter()
            .filter(|ss| ctrl.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c);

        let champ_name = ctrl.lookup.get_champion(&c)?.name.to_string();
        let mut first = true;
        for shard in shards {
            let skin = ctrl.lookup.get_skin(&shard.skin_id)?;
            if first {
                lines.push(styled_line!(LIST [
                    styled_span!(format!("{:<16}  ", format!("{}:", champ_name)); Bold Color::White),
                    styled_span!(skin.name),
                ]));
                first = false;
            } else {
                lines.push(styled_line!("{:<16}  {}", "", skin.name));
            }
        }
    }
    Ok(lines)
}

impl_text_view!(
    SkinShardsFirstSkinView,
    skin_shards_first_skin_view,
    "Skin Shards for First Skin"
);

// ============================================================================
// Disenchantable Skin Shards View
// ============================================================================

fn skin_shards_disenchantable_view(ctrl: &Controller) -> TextCreationResult {
    let skin_shards = &ctrl.manager.get_loot().recv().unwrap()?.skin_shards;
    let skins = ctrl.util.get_owned_nobase_skins().recv().unwrap()?;
    let skins_per_champ = skins.iter().fold(HashMap::new(), |mut map, skin| {
        *map.entry(skin.champ_id.clone()).or_insert(0u8) += 1;
        map
    });

    let champs_by_mastery = ctrl
        .util
        .get_champions_sorted_by_mastery(Some(12_000), None)
        .recv()
        .unwrap()?;
    let mut sorted_champs_with_skins = champs_by_mastery
        .iter()
        .filter(|cid| skins_per_champ.contains_key(cid))
        .collect::<Vec<_>>();
    sorted_champs_with_skins.reverse();

    let mut lines = vec![
        styled_line!("Shows skin shards for champs with less than 12000 mastery points and for which a skin is already owned (amount in parenthesis):"),
        styled_line!(),
    ];

    for c in sorted_champs_with_skins {
        let shards = skin_shards
            .iter()
            .filter(|ss| ctrl.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c.clone());

        let champ_name = ctrl.lookup.get_champion(c)?.name.to_string();
        let skin_count = skins_per_champ.get(c).unwrap_or(&0);
        let mut first = true;
        for shard in shards {
            let skin = ctrl.lookup.get_skin(&shard.skin_id)?;
            if first {
                lines.push(styled_line!(LIST [
                    styled_span!(format!("{:<19}  ", format!("{} ({}):", champ_name, skin_count)); Bold Color::White),
                    styled_span!(skin.name),
                ]));
                first = false;
            } else {
                lines.push(styled_line!("{:<19}  {}", "", skin.name));
            }
        }
    }
    Ok(lines)
}

impl_text_view!(
    SkinShardsDisenchantableView,
    skin_shards_disenchantable_view,
    "Disenchantable Skin Shards"
);
