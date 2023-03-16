use std::io::stdin;

use view::repl;

use crate::service::data_manager::DataManager;

mod model;
mod service;
mod view;

#[allow(dead_code)]
mod test;

fn main() {
    // test::main();

    let league_path = r"C:\Program Files\Riot Games\League of Legends\";

    match DataManager::new(league_path) {
        Ok(manager) => match repl::run(manager) {
            Ok(_) => return,
            Err(error) => println!("Error occured while running REPL:\n{:#?}\n", error),
        },
        Err(error) => println!("Error occured while initializing:\n{:#?}\n", error),
    };

    let mut s = String::new();
    println!("Press Enter to exit");
    let _ = stdin().read_line(&mut s);
}
