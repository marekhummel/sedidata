use crate::{
    impl_text_view,
    model::champion::Champion,
    styled_line,
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

    let mut lines = vec![styled_line!()];

    for (champ, mastery) in champions.iter().zip(masteries) {
        lines.push(styled_line!(
            "{:<15} ({} pts missing)",
            champ.name,
            mastery.points_to_next_level
        ));
    }
    Ok(lines)
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

    let mut lines = vec![styled_line!()];

    for (level, tokens, champ_name, upgradable) in full_info {
        let champ_name = champ_name?;
        let ready = tokens == level - 3;

        let line = if ready && upgradable {
            styled_line!(
                "{:<15} (Level {}): {}/{} tokens - READY FOR UPGRADE",
                champ_name,
                level,
                tokens,
                level - 3
            )
        } else if ready {
            styled_line!(
                "{:<15} (Level {}): {}/{} tokens - missing shard",
                champ_name,
                level,
                tokens,
                level - 3
            )
        } else {
            styled_line!("{:<15} (Level {}): {}/{} tokens", champ_name, level, tokens, level - 3)
        };

        lines.push(line);
    }
    Ok(lines)
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

    let mut lines = vec![styled_line!()];

    for c in &unplayed {
        lines.push(styled_line!("  {}", c.name));
    }

    lines.push(styled_line!());
    lines.push(styled_line!("{} champ(s) total", unplayed.len(); Cyan));
    Ok(lines)
}

impl_text_view!(
    UnplayedChampsView,
    unplayed_champs_view,
    "Unplayed Champions",
    "Unplayed Champions"
);
