use std::io::{stdin, stdout, Write};

use crossterm::{
    cursor::{position, MoveTo},
    execute,
    terminal::{
        size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, ScrollUp, SetSize,
        SetTitle,
    },
};

use crate::{
    service::{
        data_manager::{DataManager, DataRetrievalResult},
        lookup::LookupService,
        util::UtilService,
    },
    view::{
        subviews::{basic::BasicView, inventory::InventoryView, loot::LootView},
        ViewResult,
    },
};

use super::ReplError;

type CommandFunction = fn(&BasicView, &InventoryView, &LootView) -> ViewResult;
type Command<'a> = (u8, &'a str, CommandFunction);

pub fn run(manager: DataManager) -> Result<(), ReplError> {
    let lookup = get_lookup_service(&manager)?;
    let util = UtilService::new(&manager);

    let basic_view = BasicView::new(&manager);
    let inventory_view = InventoryView::new(&lookup, &util);
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

    let _ = execute!(
        stdout(),
        SetTitle("League Statistics"),
        Clear(ClearType::All),
        MoveTo(0, 0),
    );
    println!("Welcome!");
    let _ = basic_view.print_summoner();
    println!("==================================\n");
    print_options(&available_commands);
    let (cx, cy) = position()?;
    let (sx, sy) = size()?;

    loop {
        let choice = get_command(&available_commands);
        let _ = execute!(
            stdout(),
            EnterAlternateScreen,
            SetSize(sx, 250),
            MoveTo(0, 0)
        );
        match choice {
            None => break,
            Some(command) => {
                println!("~~~~~\n");
                let result = command.2(&basic_view, &inventory_view, &loot_view);
                match result {
                    Ok(_) => {
                        println!("\n~~~~~\n");
                        println!("Press Enter to go back to menu");
                        let mut s = String::new();
                        let _ = stdin().read_line(&mut s);
                        let _ = execute!(
                            stdout(),
                            LeaveAlternateScreen,
                            SetSize(sx, sy),
                            MoveTo(cx, cy - 1),
                            Clear(ClearType::FromCursorDown)
                        );
                    }
                    Err(err) => {
                        println!("Error occured: {:#?}", err);
                        break;
                    }
                }
            }
        }
    }

    let _ = execute!(
        stdout(),
        LeaveAlternateScreen,
        SetSize(sx, sy),
        MoveTo(cx, cy),
        Clear(ClearType::FromCursorDown)
    );
    println!("\nBye bye!");
    Ok(())
}

fn get_lookup_service(manager: &DataManager) -> DataRetrievalResult<LookupService> {
    let champions = manager.get_champions()?;
    let skins = manager.get_skins()?;

    Ok(LookupService::new(champions, skins))
}

fn print_options(available_commands: &Vec<Command>) {
    for (id, desc, _) in available_commands {
        println!("({id:>2})  {desc}");
    }
    println!("( 0)  Quit\n");
}

fn get_command<'a>(available_commands: &'a Vec<Command>) -> Option<&'a Command<'a>> {
    loop {
        let mut s = String::new();
        print!("> Your choice: ");
        let _ = stdout().flush();
        let _ = stdin().read_line(&mut s);

        if let Ok(choice) = s.trim().parse::<u8>() {
            if choice == 0 {
                return None;
            }

            if let Some(command) = available_commands.iter().find(|cmd| cmd.0 == choice) {
                return Some(command);
            }
        }
    }
}
