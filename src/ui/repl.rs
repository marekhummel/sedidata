use std::io::stdout;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::{
    service::{
        data_manager::{DataManager, DataRetrievalResult},
        lookup::LookupService,
        util::UtilService,
    },
    ui::{views::*, Controller, RenderContext},
};

use super::ReplError;

type ViewFactory = fn(&Controller) -> Box<dyn RenderableView>;

enum AppState {
    Menu,
    ViewingOutput(Box<dyn RenderableView>),
}

struct MenuEntry {
    description: &'static str,
    factory: Option<ViewFactory>,
}

struct App {
    menu_entries: Vec<MenuEntry>,
    selected: usize,
    should_quit: bool,
    should_refresh: bool,
    state: AppState,
    scroll_offset: u16,
}

impl App {
    fn new() -> Self {
        Self {
            menu_entries: App::get_menu_entries(),
            selected: App::get_menu_entries()
                .iter()
                .position(|e| e.factory.is_some())
                .unwrap_or(0),
            should_quit: false,
            should_refresh: false,
            state: AppState::Menu,
            scroll_offset: 0,
        }
    }

    fn is_in_menu(&self) -> bool {
        matches!(self.state, AppState::Menu)
    }

    fn next(&mut self) {
        match &self.state {
            AppState::Menu => {
                if self.menu_entries.is_empty() {
                    return;
                }
                let len = self.menu_entries.len();
                let mut i = self.selected;
                loop {
                    i = (i + 1) % len;
                    if self.menu_entries[i].factory.is_some() {
                        self.selected = i;
                        break;
                    }
                    if i == self.selected {
                        break; // no selectable entries
                    }
                }
            }
            AppState::ViewingOutput(_) => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
        }
    }

    fn previous(&mut self) {
        match &self.state {
            AppState::Menu => {
                if self.menu_entries.is_empty() {
                    return;
                }
                let len = self.menu_entries.len();
                let mut i = self.selected;
                loop {
                    i = if i == 0 { len - 1 } else { i - 1 };
                    if self.menu_entries[i].factory.is_some() {
                        self.selected = i;
                        break;
                    }
                    if i == self.selected {
                        break; // no selectable entries
                    }
                }
            }
            AppState::ViewingOutput(_) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }

    fn page_down(&mut self, amount: u16) {
        if !self.is_in_menu() {
            self.scroll_offset = self.scroll_offset.saturating_add(amount);
        }
    }

    fn page_up(&mut self, amount: u16) {
        if !self.is_in_menu() {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        }
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        // Split the provided area into a main list area and a small footer area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // main list (takes remaining space) and a small footer for refresh/quit hints
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        // Build list items; headers (factory == None) are styled and non-selectable.
        let mut items: Vec<ListItem> = Vec::with_capacity(self.menu_entries.len());
        for (i, entry) in self.menu_entries.iter().enumerate() {
            if entry.factory.is_none() {
                // Group header - cyan bold
                items.push(
                    ListItem::new(format!("━━ {} ━━", entry.description))
                        .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                );
            } else {
                // Regular menu item - subtle indicator for selected item
                let prefix = if i == self.selected { "  ► " } else { "    " };
                items.push(ListItem::new(format!("{}{}", prefix, entry.description)));
            }
        }

        let mut list_state = ListState::default();
        // Ensure selected points to a selectable entry (it should already), but guard anyway
        let sel = if self
            .menu_entries
            .get(self.selected)
            .map(|e| e.factory.is_some())
            .unwrap_or(false)
        {
            Some(self.selected)
        } else {
            // find first selectable
            self.menu_entries.iter().position(|e| e.factory.is_some())
        };
        list_state.select(sel);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .padding(ratatui::widgets::Padding::uniform(1))
                    .title("Commands (↑/↓ to navigate, Enter to select)")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            )
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
            .highlight_symbol("");

        // Render the selectable menu in the top chunk
        frame.render_stateful_widget(list, chunks[0], &mut list_state);

        // Render the footer with subtle instructions
        let footer = Paragraph::new("Refresh data: (r)    Quit: (q)")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(footer, chunks[1]);
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        manager: &mut DataManager,
    ) -> Result<(), ReplError> {
        loop {
            let lookup = App::get_lookup_service(manager)?;
            let util = UtilService::new(manager);

            loop {
                let summoner_name = manager.get_summoner().game_name.clone();

                terminal.draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(0)])
                        .split(f.size());

                    // Title with subtle welcome message
                    let title = Paragraph::new(format!(" Welcome, {}!", summoner_name))
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Cyan))
                                .title("Sedidata - LoL Special Statistics")
                                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        );
                    f.render_widget(title, chunks[0]);

                    // Render current state
                    match &self.state {
                        AppState::Menu => {
                            self.render_menu(f, chunks[1]);
                        }
                        AppState::ViewingOutput(view) => {
                            let block = Block::default()
                                .borders(ratatui::widgets::Borders::ALL)
                                .padding(ratatui::widgets::Padding::horizontal(1))
                                .title(format!(
                                    "{} (↑/↓ or PgUp/PgDown to scroll, Esc/q to return)",
                                    view.title()
                                ))
                                .title_style(
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(ratatui::style::Modifier::BOLD),
                                )
                                .border_style(Style::default().fg(Color::Cyan));

                            let rc = RenderContext {
                                frame: f,
                                area: chunks[1],
                                scroll_offset: self.scroll_offset,
                                block,
                            };
                            let _ = view.render(rc);
                        }
                    }
                })?;

                if event::poll(std::time::Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }

                        match key.code {
                            KeyCode::Char('q') if self.is_in_menu() => {
                                self.should_quit = true;
                                break;
                            }
                            KeyCode::Char('r') if self.is_in_menu() => {
                                self.should_refresh = true;
                                break;
                            }
                            KeyCode::Up => self.previous(),
                            KeyCode::Down => self.next(),
                            KeyCode::PageUp => self.page_up(10),
                            KeyCode::PageDown => self.page_down(10),
                            KeyCode::Esc | KeyCode::Char('q') if !self.is_in_menu() => {
                                self.state = AppState::Menu;
                                self.scroll_offset = 0;
                            }
                            KeyCode::Enter if self.is_in_menu() => {
                                if let Some(factory) = self.menu_entries[self.selected].factory {
                                    let ctrl = Controller {
                                        manager,
                                        lookup: &lookup,
                                        util: &util,
                                    };
                                    let view = factory(&ctrl);

                                    terminal.clear()?;
                                    self.state = AppState::ViewingOutput(view);
                                    self.scroll_offset = 0;
                                }
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

    fn get_lookup_service<'a>(manager: &'a DataManager) -> DataRetrievalResult<LookupService<'a>> {
        let champions = manager.get_champions()?;
        let skins = manager.get_skins()?;
        let masteries = manager.get_masteries()?;
        let challenges = manager.get_challenges()?;

        Ok(LookupService::new(champions, skins, masteries, challenges))
    }

    fn get_menu_entries() -> Vec<MenuEntry> {
        macro_rules! menu_entry {
            (group: $desc:expr) => {
                MenuEntry {
                    description: $desc,
                    factory: None,
                }
            };
            (item: $desc:expr, $view:ty) => {
                MenuEntry {
                    description: $desc,
                    factory: Some(|ctrl| Box::new(<$view>::new(ctrl))),
                }
            };
        }

        vec![
            // Basic
            menu_entry!(group: "Basic"),
            menu_entry!(item: "Show Summoner Info", SummonerInfoView),
            // Live game
            menu_entry!(group: "Live Game"),
            menu_entry!(item: "Champ Select Info", ChampSelectInfoView),
            // Mastery
            menu_entry!(group: "Mastery"),
            menu_entry!(item: "Level Four Champions", LevelFourChampsView),
            menu_entry!(item: "Mastery Tokens", MasteryTokensView),
            menu_entry!(item: "Unplayed Champions", UnplayedChampsView),
            // Progress
            menu_entry!(group: "Progress"),
            menu_entry!(item: "Challenges Overview", ChallengesOverviewView),
            // Inventory
            menu_entry!(group: "Inventory"),
            menu_entry!(item: "Champions Without Skin", ChampionsWithoutSkinView),
            menu_entry!(item: "Chromas Without Skin", ChromasWithoutSkinView),
            // Loot
            menu_entry!(group: "Loot"),
            menu_entry!(item: "Blue Essence Info", BlueEssenceOverviewView),
            menu_entry!(item: "Missing Champion Shards", MissingChampShardsView),
            menu_entry!(item: "Interesting Skins", InterestingSkinsView),
            menu_entry!(item: "Skin Shards for First Skin", SkinShardsFirstSkinView),
            menu_entry!(item: "Disenchantable Skin Shards", SkinShardsDisenchantableView),
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
        eprintln!("Error: {}", err);
    }

    result
}
