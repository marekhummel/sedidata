use std::io::{stdout, Read};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gag::BufferRedirect;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

use crate::{
    service::{
        data_manager::{DataManager, DataRetrievalResult},
        lookup::LookupService,
        util::UtilService,
    },
    view::{
        subviews::{
            basic::BasicView, challenges::ChallengesView, champselect::ChampSelectView, games::GamesView,
            inventory::InventoryView, loot::LootView,
        },
        ViewResult,
    },
};

use super::ReplError;

type CommandFunction =
    fn(&BasicView, &InventoryView, &LootView, &GamesView, &ChampSelectView, &ChallengesView) -> ViewResult;
type CommandEntry = (&'static str, CommandFunction);

struct App {
    commands: Vec<CommandEntry>,
    selected: usize,
    should_quit: bool,
    should_refresh: bool,
    in_output_view: bool,
    output_content: Vec<String>,
    output_title: String,
    scroll_offset: u16,
}

impl App {
    fn new() -> Self {
        Self {
            commands: App::get_commands(),
            selected: 0,
            should_quit: false,
            should_refresh: false,
            in_output_view: false,
            output_content: Vec::new(),
            output_title: String::new(),
            scroll_offset: 0,
        }
    }

    fn next(&mut self) {
        if self.in_output_view {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        } else {
            self.selected = (self.selected + 1) % self.commands.len();
        }
    }

    fn previous(&mut self) {
        if self.in_output_view {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        } else {
            self.selected = if self.selected == 0 {
                self.commands.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    fn page_down(&mut self, amount: u16) {
        if self.in_output_view {
            self.scroll_offset = self.scroll_offset.saturating_add(amount);
        }
    }

    fn page_up(&mut self, amount: u16) {
        if self.in_output_view {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        }
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        manager: &mut DataManager,
    ) -> Result<(), ReplError> {
        loop {
            let lookup = App::get_lookup_service(manager)?;
            let util = UtilService::new(manager);

            let basic_view = BasicView::new(manager);
            let inventory_view = InventoryView::new(&lookup, &util);
            let loot_view = LootView::new(manager, &lookup, &util);
            let games_view = GamesView::new(manager, &lookup);
            let champ_select_view = ChampSelectView::new(manager, &lookup);
            let challenges_view = ChallengesView::new(manager, &lookup);

            loop {
                let summoner_name = manager.get_summoner().display_name.clone();

                terminal.draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(0)])
                        .split(f.size());

                    // Title
                    let title = Paragraph::new(format!("Welcome {}!", summoner_name)).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Sedidata - LoL Special Statistics"),
                    );
                    f.render_widget(title, chunks[0]);

                    if self.in_output_view {
                        // Scrollable output view
                        let text: Vec<Line> = self.output_content.iter().map(|s| Line::from(s.as_str())).collect();
                        let paragraph = Paragraph::new(text)
                            .block(Block::default().borders(Borders::ALL).title(format!(
                                "{} (↑/↓ or PgUp/PgDown to scroll, Esc to return)",
                                self.output_title
                            )))
                            .wrap(Wrap { trim: false })
                            .scroll((self.scroll_offset, 0));
                        f.render_widget(paragraph, chunks[1]);
                    } else {
                        // Menu
                        let mut items: Vec<ListItem> =
                            self.commands.iter().map(|(desc, _)| ListItem::new(*desc)).collect();

                        items.push(ListItem::new(""));
                        items.push(ListItem::new("Refresh data (r)"));
                        items.push(ListItem::new("Quit (q)"));

                        let mut list_state = ListState::default();
                        list_state.select(Some(self.selected));

                        let list = List::new(items)
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title("Commands (↑/↓ to navigate, Enter to select, r: refresh, q: quit)"),
                            )
                            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                            .highlight_symbol(">> ");
                        f.render_stateful_widget(list, chunks[1], &mut list_state);
                    }
                })?;

                if event::poll(std::time::Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }

                        match key.code {
                            KeyCode::Char('q') if !self.in_output_view => {
                                self.should_quit = true;
                                break;
                            }
                            KeyCode::Char('r') if !self.in_output_view => {
                                self.should_refresh = true;
                                break;
                            }
                            KeyCode::Up => self.previous(),
                            KeyCode::Down => self.next(),
                            KeyCode::PageUp => self.page_up(10),
                            KeyCode::PageDown => self.page_down(10),
                            KeyCode::Esc if self.in_output_view => {
                                self.in_output_view = false;
                                self.scroll_offset = 0;
                                self.output_content.clear();
                                self.output_title.clear();
                            }
                            KeyCode::Enter if !self.in_output_view => {
                                let command = &self.commands[self.selected];

                                // Capture stdout while running the command
                                let (result, mut output_lines) = {
                                    // Temporarily exit raw mode but stay in alternate screen
                                    disable_raw_mode()?;

                                    // Capture stdout
                                    let mut buf = BufferRedirect::stdout().unwrap();

                                    let result = command.1(
                                        &basic_view,
                                        &inventory_view,
                                        &loot_view,
                                        &games_view,
                                        &champ_select_view,
                                        &challenges_view,
                                    );

                                    // Read captured output
                                    let mut captured_output = String::new();
                                    buf.read_to_string(&mut captured_output).ok();
                                    drop(buf);

                                    // Convert to lines
                                    let lines: Vec<String> = captured_output.lines().map(|s| s.to_string()).collect();

                                    // Re-enable raw mode
                                    enable_raw_mode()?;

                                    (result, lines)
                                };

                                match result {
                                    Ok(_) => {
                                        if output_lines.is_empty() {
                                            output_lines.push("Command executed successfully.".to_string());
                                            output_lines.push("(No output produced)".to_string());
                                        }
                                    }
                                    Err(err) => {
                                        output_lines =
                                            vec!["Error occurred:".to_string(), "".to_string(), format!("{:?}", err)];
                                    }
                                }

                                terminal.clear()?;
                                self.output_title = command.0.to_string();
                                self.output_content = output_lines;
                                self.in_output_view = true;
                                self.scroll_offset = 0;
                            }
                            _ => {}
                        }
                    }
                }
            }

            if self.should_quit {
                return Ok(());
            }

            if self.should_refresh {
                self.should_refresh = false;
                manager.refresh()?;
            }
        }
    }

    fn get_lookup_service(manager: &DataManager) -> DataRetrievalResult<LookupService> {
        let champions = manager.get_champions()?;
        let skins = manager.get_skins()?;
        let masteries = manager.get_masteries()?;

        Ok(LookupService::new(champions, skins, masteries))
    }

    fn get_commands() -> Vec<CommandEntry> {
        vec![
            ("Show Summoner Info", |bv, _, _, _, _, _| BasicView::print_summoner(bv)),
            ("Champions Without Skin", |_, iv, _, _, _, _| {
                InventoryView::champions_without_skin(iv)
            }),
            ("Chromas Without Skin", |_, iv, _, _, _, _| {
                InventoryView::chromas_without_skin(iv)
            }),
            ("Level Four Champions", |_, _, lv, _, _, _| {
                LootView::level_four_champs(lv)
            }),
            ("Mastery Tokens", |_, _, lv, _, _, _| LootView::mastery_tokens(lv)),
            ("Unplayed Champions", |_, _, lv, _, _, _| LootView::unplayed_champs(lv)),
            ("Blue Essence Info", |_, _, lv, _, _, _| {
                LootView::blue_essence_overview(lv)
            }),
            ("Missing Champion Shards", |_, _, lv, _, _, _| {
                LootView::missing_champ_shards(lv)
            }),
            ("Interesting Skins", |_, _, lv, _, _, _| LootView::interesting_skins(lv)),
            ("Skin Shards for First Skin", |_, _, lv, _, _, _| {
                LootView::skin_shards_first_skin(lv)
            }),
            ("Disenchantable Skin Shards", |_, _, lv, _, _, _| {
                LootView::skin_shards_disenchantable(lv)
            }),
            ("Played Games", |_, _, _, gv, _, _| GamesView::played_games(gv)),
            ("List Pentas", |_, _, _, gv, _, _| GamesView::list_pentas(gv)),
            ("Champ Select Info", |_, _, _, _, csv, _| {
                ChampSelectView::current_champ_info(csv)
            }),
            ("Challenges Overview", |_, _, _, _, _, cv| {
                ChallengesView::open_challenges_view(cv)
            }),
        ]
    }
}

pub fn run(mut manager: DataManager) -> Result<(), ReplError> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = app.run(&mut terminal, &mut manager);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = &result {
        eprintln!("Error: {:?}", err);
    }

    result
}
