use std::io::{stdin, stdout, Write};

use crate::{
    service::{data_manager::DataManager, dictionary::Dictionary},
    view::inventory::InventoryView,
};

use super::ViewError;

pub fn run(manager: DataManager) {
    let dictionary = get_dictionary(&manager);
    let inventory_view = InventoryView::new(&manager, &dictionary);

    println!("Welcome!");
    let _ = print_summoner(&manager);
    println!("===========================\n");

    loop {
        print_options();
        let choice = get_choice();
        println!("~~~");
        let result = match choice {
            1 => print_summoner(&manager),
            2 => inventory_view.champions_without_skin(),
            3 => inventory_view.chromas_without_skin(),
            // 8 => {}
            9 => break,
            _ => unreachable!(),
        };

        if let Err(err) = result {
            println!("Error occured: {:#?}", err);
            return;
        }

        println!("\n----------------------------------\n");
    }
}

fn print_summoner(manager: &DataManager) -> Result<(), ViewError> {
    let summoner = manager.get_summoner();
    println!("{:?}\n", summoner);
    Ok(())
}

fn get_dictionary(manager: &DataManager) -> Dictionary {
    let champions = manager.get_champions().unwrap();
    let skins = manager.get_skins().unwrap();

    Dictionary::new(champions, skins)
}

fn print_options() {
    println!("(1) Summoner Info");
    println!("(2) Champions Without A Skin");
    println!("(3) Chromas Without A Skin");
    // println!("(8) Refresh");
    println!("(9) Quit");
}

fn get_choice() -> u8 {
    loop {
        let mut s = String::new();
        print!("Your choice: ");
        let _ = stdout().flush();
        let _ = stdin().read_line(&mut s);

        if let Ok(choice) = s.trim().parse::<u8>() {
            return choice;
        }
    }
}
