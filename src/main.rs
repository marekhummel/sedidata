use std::io::stdin;

use ui::repl;

use crate::service::data_manager::DataManager;

mod model;
mod service;
mod ui;

fn main() {
    match DataManager::new(true, true) {
        Ok(manager) => match repl::run(manager) {
            Ok(_) => return,
            Err(error) => println!("Error occured while running REPL:\n{}\n", error),
        },
        Err(error) => println!("Error occured while initializing:\n{}\n", error),
    };

    let mut s = String::new();
    println!("Press Enter to exit");
    let _ = stdin().read_line(&mut s);
}
