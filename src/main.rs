use std::io::stdin;

use view::repl;

use crate::service::data_manager::DataManager;

mod model;
mod service;
mod view;

// #[allow(dead_code)]
// mod test;

fn main() {
    // test::main();
    // return;

    match DataManager::new(true) {
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
