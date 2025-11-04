use crate::{service::data_manager::DataManager, view::ViewResult};

pub struct BasicView<'a> {
    manager: &'a DataManager,
}

impl<'a> BasicView<'a> {
    pub fn new<'b: 'a>(manager: &'b DataManager) -> Self {
        Self { manager }
    }

    pub fn print_summoner(&self) -> ViewResult {
        let summoner = self.manager.get_summoner();

        println!("Summoner Information:");
        println!("---------------------");
        println!("Game Name: {}", summoner.game_name);
        println!("Tag Line: {}", summoner.tag_line);
        println!("Level: {}", summoner.level);
        println!();
        println!("ID: {}", summoner.id);
        println!("PUUID: {}", summoner.puuid);
        Ok(())
    }
}
