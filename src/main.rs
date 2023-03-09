use crate::{controller::data_manager::DataManager, model::champion};

mod controller;
mod gameapi;
mod model;

fn main() {
    let league_path = r"C:\Program Files\Riot Games\League of Legends\";
    let manager_res = DataManager::new(league_path);

    if let Err(error) = manager_res {
        println!("{:?}", error);
        return;
    }

    let mut manager = manager_res.unwrap();
    let summoner = manager.get_summoner();
    println!("{:?}\n", summoner);

    // let masteries = manager.get_masteries();
    // println!("{:?}", masteries);

    // let champions = manager.get_champions();
    // println!("{:?}", champions);

    // let skins = manager.get_skins();
    // println!("{:?}", skins);

    // let chromas = manager.get_chromas();
    // println!("{:?}", chromas);

    let loot = manager.get_loot();
    // println!("{:#?}", loot.unwrap().ignored);
}
