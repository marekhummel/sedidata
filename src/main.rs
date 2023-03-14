use crate::{
    model::ids::ChampionId,
    service::{data_manager::DataManager, dictionary::Dictionary},
};

mod model;
mod service;
mod test;
mod view;

fn main() {
    // test::main();
    let league_path = r"C:\Program Files\Riot Games\League of Legends\";
    let manager_res = DataManager::new(league_path);

    if let Err(error) = manager_res {
        println!("{:?}", error);
        return;
    }

    let mut manager = manager_res.unwrap();
    let summoner = manager.get_summoner();
    println!("{:?}\n", summoner);

    let champions = manager.get_champions().unwrap();
    let skins = manager.get_skins().unwrap();
    let chromas = manager.get_chromas().unwrap();
    println!("{:?} champions", champions.len());
    println!("{:?} skins", skins.len());
    println!("{:?} chromas", chromas.len());

    let masteries = manager.get_masteries().unwrap();
    println!("{:?} masteries", masteries.len());
    let loot = manager.get_loot().unwrap();
    println!("{:#?}", loot.credits);

    let dictionary = Dictionary::new(champions, skins);
    println!(
        "Champ #32 is {:?}",
        dictionary.get_champion(32.into()).map(|c| c.name.as_str())
    );
    println!(
        "Skin #238011 is {:?}",
        dictionary.get_skin(238011.into()).map(|s| s.name.as_str())
    );
}
