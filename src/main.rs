use std::io::stdin;

use clap::Parser;
use ui::repl;

use crate::service::data_manager::DataManager;

mod model;
mod service;
mod ui;

/// League of Legends data viewer and analyzer
#[derive(Parser, Debug)]
#[command(name = "sedidata")]
#[command(version, about, long_about = None)]
struct Args {
    /// Load data from local JSON files instead of fetching from the game client
    #[arg(short = 'l', long = "load-local")]
    load_local_json_files: bool,

    /// Store API responses to JSON files for debugging/testing
    #[arg(short = 's', long = "store-responses")]
    store_responses: bool,
}

fn main() {
    let args = Args::parse();

    match DataManager::new(args.load_local_json_files, args.store_responses) {
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
