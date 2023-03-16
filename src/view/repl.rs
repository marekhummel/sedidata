use std::io::{self, stdin, stdout, Write};

use crossterm::{
    cursor::{position, MoveTo},
    execute,
    terminal::{
        size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetSize, SetTitle,
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
type CommandEntry<'a> = (u8, &'a str, CommandFunction);

pub fn run(mut manager: DataManager) -> Result<(), ReplError> {
    let _ = execute!(
        stdout(),
        SetTitle("League Statistics"),
        Clear(ClearType::All),
        MoveTo(0, 0),
    );
    println!("Welcome!");

    loop {
        let lookup = get_lookup_service(&manager)?;
        let util = UtilService::new(&manager);

        let basic_view = BasicView::new(&manager);
        let inventory_view = InventoryView::new(&lookup, &util);
        let loot_view = LootView::new(&manager, &lookup, &util);

        let available_commands = get_commands();

        basic_view.print_summoner()?;
        println!("==================================\n");
        print_options(&available_commands);
        let (cx, cy) = position()?;
        let (sx, sy) = size()?;

        loop {
            let choice = get_command(&available_commands);
            execute!(
                stdout(),
                EnterAlternateScreen,
                SetSize(sx, 250),
                MoveTo(0, 0)
            )?;
            match choice {
                Command::Execute(command) => {
                    println!("~~~~~\n");
                    let result = command.2(&basic_view, &inventory_view, &loot_view);
                    match result {
                        Ok(_) => {
                            println!("\n~~~~~\n");
                            println!("Press Enter to go back to menu");
                            let mut s = String::new();
                            let _ = stdin().read_line(&mut s);
                            reset_screen_buffer(sx, sy, cx, cy)?;
                        }
                        Err(err) => {
                            reset_screen_buffer(sx, sy, cx, cy)?;
                            return Err(err.into());
                        }
                    }
                }
                Command::Refresh => {
                    reset_screen_buffer(sx, sy, 0, 2)?;
                    break;
                }
                Command::Quit => {
                    reset_screen_buffer(sx, sy, cx, cy)?;
                    println!("\nBye bye!");
                    return Ok(());
                }
            }
        }

        manager.refresh()?;
    }
}

fn get_lookup_service(manager: &DataManager) -> DataRetrievalResult<LookupService> {
    let champions = manager.get_champions()?;
    let skins = manager.get_skins()?;

    Ok(LookupService::new(champions, skins))
}

fn get_commands<'a>() -> Vec<CommandEntry<'a>> {
    vec![
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
    ]
}

fn print_options(available_commands: &Vec<CommandEntry>) {
    for (id, desc, _) in available_commands {
        println!("({id:>2})  {desc}");
    }
    println!("\n(r)  Refresh data");
    println!("(q)  Quit\n");
}

fn get_command<'a>(available_commands: &'a Vec<CommandEntry>) -> Command<'a> {
    loop {
        let mut s = String::new();
        print!("> Your choice: ");
        let _ = stdout().flush();
        let _ = stdin().read_line(&mut s);

        match s.trim() {
            "q" => return Command::Quit,
            "r" => return Command::Refresh,
            ts => {
                if let Ok(choice) = ts.parse::<u8>() {
                    if let Some(command) = available_commands.iter().find(|cmd| cmd.0 == choice) {
                        return Command::Execute(command);
                    }
                }
            }
        };
    }
}

fn reset_screen_buffer(sx: u16, sy: u16, cx: u16, cy: u16) -> Result<(), io::Error> {
    execute!(
        stdout(),
        LeaveAlternateScreen,
        SetSize(sx, sy),
        MoveTo(cx, cy - 1),
        Clear(ClearType::FromCursorDown)
    )
}

enum Command<'a> {
    Quit,
    Refresh,
    Execute(&'a CommandEntry<'a>),
}
