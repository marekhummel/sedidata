use crate::{
    impl_text_view,
    model::champion::Champion,
    ui::{Controller, TextCreationResult},
};

// ============================================================================
// Level Four Champions View
// ============================================================================

fn level_four_champs_view(ctrl: &Controller) -> TextCreationResult {
    let mut masteries = ctrl.util.get_masteries_with_level(vec![4])?;
    masteries.sort_by_key(|m| m.points_to_next_level);

    let champions = masteries
        .iter()
        .map(|m| ctrl.lookup.get_champion(&m.champ_id))
        .collect::<Result<Vec<&Champion>, _>>()?;

    let mut result = String::from("Champions that are mastery level 4:\n\n");
    for (champ, mastery) in champions.iter().zip(masteries) {
        result.push_str(&format!(
            "{:<15} ({} pts missing)\n",
            champ.name, mastery.points_to_next_level
        ));
    }
    Ok(result)
}

impl_text_view!(
    LevelFourChampsView,
    level_four_champs_view,
    "Level Four Champions",
    "Level Four Champions"
);

// ============================================================================
// Mastery Tokens View
// ============================================================================

fn mastery_tokens_view(ctrl: &Controller) -> TextCreationResult {
    let maxed_masteries = ctrl.util.get_masteries_with_level(vec![5, 6])?;
    let champ_shards_set = ctrl.util.get_champ_shard_set()?;

    let mut full_info = maxed_masteries
        .into_iter()
        .map(|m| {
            (
                m.level,
                m.tokens.unwrap_or(0),
                ctrl.lookup.get_champion(&m.champ_id).map(|c| c.name.to_string()),
                champ_shards_set.contains(&m.champ_id),
            )
        })
        .collect::<Vec<_>>();

    full_info.sort_by_key(|(level, tokens, champ_name, upgradable)| {
        (
            -(*level as i16),
            -(*tokens as i16),
            *upgradable,
            champ_name.as_ref().map_or("".to_string(), |s| s.clone()),
        )
    });

    let mut result = String::from("Mastery tokens and if they can be upgraded:\n\n");
    for (level, tokens, champ_name, upgradable) in full_info {
        let champ_name = champ_name?;
        result.push_str(&format!(
            "{:<15} (Level {}): {}/{} tokens{}\n",
            champ_name,
            level,
            tokens,
            level - 3,
            match (tokens == level - 3, upgradable) {
                (true, true) => " - READY FOR UPGRADE",
                (true, false) => " - MISSING SHARD",
                _ => "",
            }
        ));
    }
    Ok(result)
}

impl_text_view!(
    MasteryTokensView,
    mastery_tokens_view,
    "Mastery Tokens",
    "Mastery Tokens"
);

// ============================================================================
// Unplayed Champions View
// ============================================================================

fn unplayed_champs_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.manager.get_champions()?;
    let played_champs = ctrl.util.get_played_champions_set()?;

    let mut unplayed = champs
        .iter()
        .filter(|c| !played_champs.contains(&c.id))
        .collect::<Vec<_>>();
    unplayed.sort_by_key(|c| c.name.as_str());

    let mut result = String::from("Champions with 0 mastery points:\n\n");
    for c in &unplayed {
        result.push_str(&format!("{}\n", c.name));
    }
    result.push_str(&format!("\n{} champ(s) total", unplayed.len()));
    Ok(result)
}

impl_text_view!(
    UnplayedChampsView,
    unplayed_champs_view,
    "Unplayed Champions",
    "Unplayed Champions"
);
