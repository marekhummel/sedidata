use crate::{
    impl_text_view,
    model::games::{Game, QueueType},
    ui::{Controller, TextCreationResult},
};
use itertools::Itertools;
use std::collections::HashMap;

// ============================================================================
// Played Games View
// ============================================================================

fn played_games_view(ctrl: &Controller) -> TextCreationResult {
    let games = ctrl.manager.get_game_stats()?;
    let games_by_queue: HashMap<&QueueType, Vec<&Game>> = games.iter().fold(HashMap::new(), |mut map, game| {
        map.entry(&game.queue).or_default().push(game);
        map
    });

    let mut result = String::from("Games winrate since season 8:\n\n");
    for (queue, games) in games_by_queue {
        result.push_str(&format!("Queue: {:?}\n", queue));

        let won = games.iter().filter(|g| g.victory).count();
        let total = games.len();
        result.push_str(&format!("  Played:  {}\n", total));
        result.push_str(&format!(
            "  Won:     {} (wr: {:.3}%)\n",
            won,
            (won as f32) / (total as f32) * 100.0
        ));
        result.push('\n');
    }
    Ok(result)
}

impl_text_view!(PlayedGamesView, played_games_view, "Played Games", "Played Games");

// ============================================================================
// List Pentas View
// ============================================================================

fn list_pentas_view(ctrl: &Controller) -> TextCreationResult {
    let games = ctrl.manager.get_game_stats()?;
    let mut penta_games = games.iter().filter(|g| g.stats.pentas > 0).collect::<Vec<_>>();
    penta_games.sort_by_key(|g| g.timestamp);
    penta_games.reverse();

    let mut result = String::from("Penta kills since season 8 (only on rift, not aram):\n\n");
    let mut last_season = None;
    let mut cntr = games.iter().map(|g| g.stats.pentas).sum::<u16>();
    for g in penta_games {
        match last_season {
            Some(season) if season != g.season => result.push_str(&format!("\nSeason {}\n", g.season)),
            None => result.push_str(&format!("Season {}\n", g.season)),
            _ => {}
        }

        let champ = ctrl.lookup.get_champion(&g.champ_id)?;
        for _ in 0..g.stats.pentas {
            result.push_str(&format!(
                "#{:0>2}: [{}] {} in {:?}\n",
                cntr,
                g.timestamp.format("%d.%m.%Y %H:%M"),
                champ.name,
                g.queue
            ));
            cntr -= 1;
        }

        last_season = Some(g.season);
    }
    Ok(result)
}

impl_text_view!(ListPentasView, list_pentas_view, "List Pentas", "List Pentas");

// ============================================================================
// Challenges Overview View
// ============================================================================

fn challenges_overview_view(ctrl: &Controller) -> TextCreationResult {
    let mut challenges = ctrl.manager.get_challenges()?.to_vec();
    challenges.retain(|c| !c.is_capstone && !c.is_completed() && c.category != "LEGACY");
    challenges.sort_by_key(|c| (c.category.clone(), -(c.points_to_next() as i16)));

    let mut result = String::new();
    for (category, iterator) in &challenges.iter().chunk_by(|c| c.category.clone()) {
        result.push_str(&format!("\nCategory: {}\n", category));
        for challenge in iterator {
            result.push_str(&format!(
                "({: >3}) {: <30}: {} ([{}]) ({}/{})\n",
                challenge.points_to_next(),
                challenge.name,
                challenge.description,
                challenge.gamemodes.join(", "),
                challenge.current_value,
                challenge.threshold_value
            ));
        }
    }
    Ok(result)
}

impl_text_view!(
    ChallengesOverviewView,
    challenges_overview_view,
    "Challenges Overview",
    "Challenges Overview"
);
