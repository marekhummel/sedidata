use crate::{
    impl_text_view,
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
    let loot = ctrl.manager.get_loot()?;
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

    let result = format!(
        "Current BE: {}\nConvertable BE: {}\nConvertable BE (Keep one shard per champ): {}\nConvertable BE (Keep two shards per champ): {}",
        be, convertable, keep1, keep2
    );
    Ok(result)
}

impl_text_view!(
    BlueEssenceOverviewView,
    blue_essence_overview_view,
    "Blue Essence Info",
    "Blue Essence Info"
);

// ============================================================================
// Missing Champion Shards View
// ============================================================================

fn missing_champ_shards_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.manager.get_champions()?;
    let loot = ctrl.manager.get_loot()?;
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

    let mut result = String::from("Champions for which no champ shard is owned:\n\n");
    for c in &missing_cs {
        result.push_str(&format!("{}\n", c.name));
    }
    result.push_str(&format!("\n{} champ(s) total", missing_cs.len()));
    Ok(result)
}

impl_text_view!(
    MissingChampShardsView,
    missing_champ_shards_view,
    "Missing Champion Shards",
    "Missing Champion Shards"
);

// ============================================================================
// Interesting Skins View
// ============================================================================

fn interesting_skins_view(ctrl: &Controller) -> TextCreationResult {
    let sorted_champs = ctrl.util.get_champions_sorted_by_mastery(None, Some(10_000))?;
    let skin_shards = &ctrl.manager.get_loot()?.skin_shards;

    let mut result =
        String::from("Owned skin shards for champs with 10k or more mastery points (sorted by mastery points):\n\n");
    for c in sorted_champs {
        let shards = skin_shards
            .iter()
            .filter(|ss| ctrl.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c);

        let mut prefix = ctrl.lookup.get_champion(&c)?.name.to_string();
        prefix.push(':');
        for shard in shards {
            let skin_name = ctrl.lookup.get_skin(&shard.skin_id)?.name.as_str();
            result.push_str(&format!("{:<16}  {}\n", prefix, skin_name));
            prefix = "".to_string();
        }
    }
    Ok(result)
}

impl_text_view!(
    InterestingSkinsView,
    interesting_skins_view,
    "Interesting Skins",
    "Interesting Skins"
);

// ============================================================================
// Skin Shards for First Skin View
// ============================================================================

fn skin_shards_first_skin_view(ctrl: &Controller) -> TextCreationResult {
    let skin_shards = &ctrl.manager.get_loot()?.skin_shards;
    let skins = ctrl.util.get_owned_nobase_skins()?;
    let champs_with_skin = skins.iter().map(|s| s.champ_id.clone()).collect::<HashSet<_>>();

    let sorted_champs = ctrl.util.get_champions_sorted_by_mastery(None, None)?;
    let champs_no_skin = sorted_champs.into_iter().filter(|cid| !champs_with_skin.contains(cid));

    let mut result =
        String::from("Shows skin shards which would be the first skin for the champ (sorted by mastery points):\n\n");
    for c in champs_no_skin {
        let shards = skin_shards
            .iter()
            .filter(|ss| ctrl.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c);

        let mut prefix = ctrl.lookup.get_champion(&c)?.name.to_string();
        prefix.push(':');
        for shard in shards {
            let skin_name = ctrl.lookup.get_skin(&shard.skin_id)?.name.as_str();
            result.push_str(&format!("{:<16}  {}\n", prefix, skin_name));
            prefix = "".to_string();
        }
    }
    Ok(result)
}

impl_text_view!(
    SkinShardsFirstSkinView,
    skin_shards_first_skin_view,
    "Skin Shards for First Skin",
    "Skin Shards for First Skin"
);

// ============================================================================
// Disenchantable Skin Shards View
// ============================================================================

fn skin_shards_disenchantable_view(ctrl: &Controller) -> TextCreationResult {
    let skin_shards = &ctrl.manager.get_loot()?.skin_shards;
    let skins = ctrl.util.get_owned_nobase_skins()?;
    let skins_per_champ = skins.iter().fold(HashMap::new(), |mut map, skin| {
        *map.entry(skin.champ_id.clone()).or_insert(0u8) += 1;
        map
    });

    let champs_by_mastery = ctrl.util.get_champions_sorted_by_mastery(Some(12_000), None)?;
    let mut sorted_champs_with_skins = champs_by_mastery
        .iter()
        .filter(|cid| skins_per_champ.contains_key(cid))
        .collect::<Vec<_>>();
    sorted_champs_with_skins.reverse();

    let mut result = String::from("Shows skin shards for champs with less than 12000 mastery points and for which a skin is already owned (amount in parenthesis):\n\n");
    for c in sorted_champs_with_skins {
        let shards = skin_shards
            .iter()
            .filter(|ss| ctrl.lookup.get_skin(&ss.skin_id).unwrap().champ_id == c.clone());

        let mut champ_prefix = ctrl.lookup.get_champion(c)?.name.to_string();
        champ_prefix.push_str(&format!(" ({})", skins_per_champ.get(c).unwrap_or(&0)));
        champ_prefix.push(':');
        for shard in shards {
            let skin_name = ctrl.lookup.get_skin(&shard.skin_id)?.name.as_str();
            result.push_str(&format!("{:<19}  {}\n", champ_prefix, skin_name));
            champ_prefix = "".to_string();
        }
    }
    Ok(result)
}

impl_text_view!(
    SkinShardsDisenchantableView,
    skin_shards_disenchantable_view,
    "Disenchantable Skin Shards",
    "Disenchantable Skin Shards"
);
