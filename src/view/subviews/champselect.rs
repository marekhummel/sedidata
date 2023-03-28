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
                self.print_selectable_champ(&current_champ, true)?;

                println!("\nBenched Champions:");
                for bench_champ in champ_select_info.benched_champs {
                    self.print_selectable_champ(&bench_champ.id, bench_champ.selectable)?;
                }

                println!("\nTradable Champions:");
                for team_champ in champ_select_info.team_champs {
                    self.print_selectable_champ(&team_champ.id, team_champ.selectable)?;
                }
            }
            None => println!("Not in champ select!"),
        };

        Ok(())
    }

    fn print_selectable_champ(&self, champ: &ChampionId, selectable: bool) -> ViewResult {
        let champion = self.lookup.get_champion(&champ)?;
        let mastery = self.lookup.get_mastery(&champ)?;
        print!(
            "  {:<16}  Level {} ({} pts)",
            format!("{}:", champion.name),
            mastery.level,
            mastery.points
        );
        if !selectable {
            print!(" - not owned!");
        }
        println!();
        Ok(())
    }
}
