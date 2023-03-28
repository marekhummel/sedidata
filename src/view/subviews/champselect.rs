use crate::{
    model::ids::ChampionId,
    service::{data_manager::DataManager, lookup::LookupService},
    view::ViewResult,
};

pub struct ChampSelectView<'a, 'b: 'a> {
    manager: &'a DataManager,
    lookup: &'a LookupService<'b>,
}

impl<'a, 'b> ChampSelectView<'a, 'b> {
    pub fn new(manager: &'b DataManager, lookup: &'b LookupService) -> Self {
        Self { manager, lookup }
    }

    pub fn current_champ_info(&self) -> ViewResult {
        match self.manager.get_champ_select_info()? {
            Some(champ_select_info) => {
                println!("Currently selected champ:");
                let current_champ = champ_select_info.current_champ_id;
                self.print_selectable_champ(&current_champ)?;

                println!("\nBenched Champions:");
                for bench_champ in champ_select_info.benched_champs {
                    self.print_selectable_champ(&bench_champ)?;
                }

                println!("\nTradable Champions:");
                for team_champ in champ_select_info.team_champs {
                    self.print_selectable_champ(&team_champ)?;
                }
            }
            None => println!("Not in champ select!"),
        };

        Ok(())
    }

    fn print_selectable_champ(&self, champ: &ChampionId) -> ViewResult {
        let champion = self.lookup.get_champion(&champ)?;
        print!("  {:<16}", format!("{}:", champion.name));
        match champion.owned {
            true => match self.lookup.get_mastery(&champ) {
                Ok(mastery) => {
                    print!("  Level {}", mastery.level);

                    match mastery.tokens {
                        Some(tokens) => print!(
                            " ({}/{} tokens, {} pts)",
                            tokens,
                            mastery.level - 3,
                            mastery.points
                        ),
                        None => print!(" ({} pts)", mastery.points),
                    }
                }
                Err(_) => print!(" Level 0 (not played!)"),
            },
            false => print!(" not owned!"),
        }

        println!();
        Ok(())
    }
}
