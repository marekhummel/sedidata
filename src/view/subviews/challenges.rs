use crate::{
    model::ids::ChampionId,
    service::{data_manager::DataManager, lookup::LookupService},
    view::ViewResult,
};

pub struct ChallengesView<'a, 'b: 'a> {
    manager: &'a DataManager,
    lookup: &'a LookupService<'b>,
}

impl<'a, 'b> ChallengesView<'a, 'b> {
    pub fn new(manager: &'b DataManager, lookup: &'b LookupService) -> Self {
        Self { manager, lookup }
    }

    pub fn open_challenges_view(&self) -> ViewResult {
        let challenge_categories = self.manager.get_challenges()?;

        println!("Challenges Overview:");
        for category in challenge_categories {
            println!("Category: {}", category.name);
            // for challenge in &category.children {
            //     println!("  Challenge: {}", challenge.name);
            // }
        }

        //     Some(champ_select_info) => {
        //         println!("Currently selected champ:");
        //         let current_champ = champ_select_info.current_champ_id;
        //         self.print_selectable_champ(&current_champ)?;

        //         println!("\nBenched Champions:");
        //         for bench_champ in champ_select_info.benched_champs {
        //             self.print_selectable_champ(&bench_champ)?;
        //         }

        //         println!("\nTradable Champions:");
        //         for team_champ in champ_select_info.team_champs {
        //             self.print_selectable_champ(&team_champ)?;
        //         }
        //     }
        //     None => println!("Not in champ select!"),
        // };

        Ok(())
    }
}
