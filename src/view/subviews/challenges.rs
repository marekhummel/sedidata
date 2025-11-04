use crate::{
    service::{data_manager::DataManager, lookup::LookupService},
    view::ViewResult,
};

use itertools::Itertools;

pub struct ChallengesView<'a, 'b: 'a> {
    manager: &'a DataManager,
    _lookup: &'a LookupService<'b>,
}

impl<'a, 'b> ChallengesView<'a, 'b> {
    pub fn new(manager: &'a DataManager, lookup: &'b LookupService) -> Self {
        Self {
            manager,
            _lookup: lookup,
        }
    }

    pub fn open_challenges_view(&self) -> ViewResult {
        let mut challenges = self.manager.get_challenges()?.to_vec();
        challenges.retain(|c| !c.is_capstone && !c.is_completed() && c.category != "LEGACY");
        challenges.sort_by_key(|c| (c.category.clone(), -(c.points_to_next() as i16)));

        for (category, iterator) in &challenges.iter().chunk_by(|c| c.category.clone()) {
            println!("\nCategory: {}", category);
            for challenge in iterator {
                println!(
                    "({: >3}) {: <30}: {} ([{}]) ({}/{})",
                    challenge.points_to_next(),
                    challenge.name,
                    challenge.description,
                    challenge.gamemodes.join(", "),
                    challenge.current_value,
                    challenge.threshold_value
                );
            }
        }
        Ok(())
    }
}
