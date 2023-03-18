use crate::{
    service::{data_manager::DataManager, lookup::LookupService},
    view::ViewResult,
};

pub struct GamesView<'a, 'b: 'a> {
    manager: &'a DataManager,
    lookup: &'a LookupService<'b>,
}

impl<'a, 'b> GamesView<'a, 'b> {
    pub fn new(manager: &'b DataManager, lookup: &'b LookupService) -> Self {
        Self { manager, lookup }
    }

    pub fn played_games(&self) -> ViewResult {
        println!("Games played in season 13:\n");

        let games = self.manager.get_game_stats()?;

        print!("{}", games.len());
        Ok(())
    }

    pub fn list_pentas(&self) -> ViewResult {
        println!("Penta kills since season 8 (only on rift, not aram):\n");

        let games = self.manager.get_game_stats()?;
        let mut penta_games = games
            .iter()
            .filter(|g| g.stats.pentas > 0)
            .collect::<Vec<_>>();
        penta_games.sort_by_key(|g| g.timestamp);
        penta_games.reverse();

        let mut last_season = None;
        let mut cntr = games.iter().map(|g| g.stats.pentas).sum::<u16>();
        for g in penta_games {
            match last_season {
                Some(season) if season != g.season => println!("\nSeason {}", g.season),
                None => println!("Season {}", g.season),
                _ => {}
            }

            let champ = self.lookup.get_champion(&g.champ_id)?;
            for _ in 0..g.stats.pentas {
                println!(
                    "#{:0>2}: [{}] {} in {:?}",
                    cntr,
                    g.timestamp.format("%d.%m.%Y %H:%M"),
                    champ.name,
                    g.queue
                );
                cntr -= 1;
            }

            last_season = Some(g.season);
        }
        Ok(())
    }
}
