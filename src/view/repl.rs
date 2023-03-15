use std::io::{stdin, stdout, Write};

use crate::{
    service::{data_manager::DataManager, lookup::LookupService, util::UtilService},
    view::subviews::{basic::BasicView, inventory::InventoryView, loot::LootView},
};

pub fn run(manager: DataManager) {
    let lookup = get_lookup_service(&manager);
    let util = UtilService::new(&manager);

    let basic_view = BasicView::new(&manager, &lookup, &util);
    let inventory_view = InventoryView::new(&manager, &lookup, &util);
    let loot_view = LootView::new(&manager, &lookup, &util);

    println!("Welcome!");
    let _ = basic_view.print_summoner();
    println!("===========================\n");

    loop {
        print_options();
        let choice = get_choice();
        println!("~~~");
        let result = match choice {
            0 => break,
            1 => basic_view.print_summoner(),
            10 => inventory_view.champions_without_skin(),
            11 => inventory_view.chromas_without_skin(),
            // 8 => manager.refresh().map_err(|err| err.into()),
            20 => loot_view.level_four_champs(),
            21 => loot_view.mastery_tokens(),
            _ => unreachable!(),
        };

        if let Err(err) = result {
            println!("Error occured: {:#?}", err);
            return;
        }

        println!("\n----------------------------------\n");
    }
}

fn get_lookup_service(manager: &DataManager) -> LookupService {
    let champions = manager.get_champions().unwrap();
    let skins = manager.get_skins().unwrap();

    LookupService::new(champions, skins)
}

fn print_options() {
    println!("(1) Summoner Info");
    println!("(10) Champions Without A Skin");
    println!("(11) Chromas Without A Skin");
    println!("(20) Level Four Champions");
    println!("(21) Mastery Token Info");
    // println!("(8) Refresh");
    println!("(0) Quit");
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
