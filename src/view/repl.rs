use std::io::{stdin, stdout, Write};

use crate::{
    service::{data_manager::DataManager, lookup::LookupService, util::UtilService},
    view::{
        subviews::{basic::BasicView, inventory::InventoryView, loot::LootView},
        ViewResult,
    },
};

type CommandFunction = fn(&BasicView, &InventoryView, &LootView) -> ViewResult;
type Command<'a> = (u8, &'a str, CommandFunction);

pub fn run(manager: DataManager) {
    let lookup = get_lookup_service(&manager);
    let util = UtilService::new(&manager);

    let basic_view = BasicView::new(&manager);
    let inventory_view = InventoryView::new(&manager, &lookup, &util);
    let loot_view = LootView::new(&manager, &lookup, &util);

    let available_commands: Vec<Command> = vec![
        (1, "Show Summoner", |bv, _, _| BasicView::print_summoner(bv)),
        (10, "Champions Without Skin", |_, iv, _| {
            InventoryView::champions_without_skin(iv)
        }),
        (11, "Chromas Without Skin", |_, iv, _| {
            InventoryView::chromas_without_skin(iv)
        }),
        (20, "Level Four Champions", |_, _, lv| {
            LootView::level_four_champs(lv)
        }),
        (21, "Mastery Tokens", |_, _, lv| {
            LootView::mastery_tokens(lv)
        }),
        (22, "Unplayed Champions", |_, _, lv| {
            LootView::unplayed_champs(lv)
        }),
        (23, "Blue Essence Info", |_, _, lv| {
            LootView::blue_essence_overview(lv)
        }),
        (24, "Missing Champion Shards", |_, _, lv| {
            LootView::missing_champ_shards(lv)
        }),
        (25, "Interesting Skins", |_, _, lv| {
            LootView::interesting_skins(lv)
        }),
        (26, "Skin Shards for First Skin", |_, _, lv| {
            LootView::skin_shards_first_skin(lv)
        }),
        (27, "Disenchantable Skin Shards", |_, _, lv| {
            LootView::skin_shards_disenchantable(lv)
        }),
    ];

    println!("Welcome!");
    let _ = basic_view.print_summoner();
    println!("===========================\n");

    loop {
        print_options(&available_commands);
        let choice = get_choice();
        if choice == 0 {
            break;
        }

        println!("~~~\n");
        let command = available_commands
            .iter()
            .find(|cmd| cmd.0 == choice)
            .unwrap();
        let result = command.2(&basic_view, &inventory_view, &loot_view);
        if let Err(err) = result {
            println!("Error occured: {:#?}", err);
            return;
        }

        println!("\n\n\n---------------------------------------------------\n");
    }
}

fn get_lookup_service(manager: &DataManager) -> LookupService {
    let champions = manager.get_champions().unwrap();
    let skins = manager.get_skins().unwrap();

    LookupService::new(champions, skins)
}

fn print_options(available_commands: &Vec<Command>) {
    for (id, desc, _) in available_commands {
        println!("({id:>2})  {desc}");
    }
    println!("( 0)  Quit");
}

fn get_choice() -> u8 {
    loop {
        let mut s = String::new();
        print!("> Your choice: ");
        let _ = stdout().flush();
        let _ = stdin().read_line(&mut s);

        if let Ok(choice) = s.trim().parse::<u8>() {
            return choice;
        }
    }
}
