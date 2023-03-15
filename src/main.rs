use view::repl;

use crate::service::data_manager::DataManager;

mod model;
mod service;
mod view;

fn main() {
    let league_path = r"C:\Program Files\Riot Games\League of Legends\";

    match DataManager::new(league_path) {
        Ok(manager) => repl::run(manager),
        Err(error) => println!("Error occured while initializing:\n{:#?}", error),
    };
}
