use view::repl;

use crate::service::data_manager::DataManager;

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

    let manager = manager_res.unwrap();
    repl::run(manager);

    // let chromas = manager.get_chromas().unwrap();
    // let masteries = manager.get_masteries().unwrap();
    // let loot = manager.get_loot().unwrap();
}
