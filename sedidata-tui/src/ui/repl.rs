use std::{
    io::stdout,
    sync::{Arc, Mutex},
    time::Instant,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use crate::{
    service::{
        data_manager::{DataManager, DataRetrievalResult},
        lookup::LookupService,
        util::UtilService,
    },
    ui::{menu::Menu, views::*, Controller, RenderContext},
};

use super::ReplError;

enum AppState {
    Menu,
    ViewingOutput(Box<dyn RenderableView>),
    Error(String),
}

struct App {
    state: AppState,
    menu: Menu,
    should_quit: bool,
    should_refresh: bool,
    scroll_offset: u16,
    pressed_keys: Vec<KeyCode>,
    last_refresh: Option<Instant>,
    panic_flag: Arc<Mutex<Option<String>>>,
}

impl App {
    fn new(panic_flag: Arc<Mutex<Option<String>>>) -> Self {
        Self {
            menu: Menu::new(),
            should_quit: false,
            should_refresh: false,
            state: AppState::Menu,
            scroll_offset: 0,
            pressed_keys: Vec::new(),
            last_refresh: None,
            panic_flag,
        }
    }

    fn is_in_menu(&self) -> bool {
        matches!(self.state, AppState::Menu)
    }

    fn is_in_subview(&self) -> bool {
        matches!(self.state, AppState::ViewingOutput(_))
    }

    fn next(&mut self) {
        match &self.state {
            AppState::Menu => {
                self.menu.next();
            }
            AppState::ViewingOutput(_) | AppState::Error(_) => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
        }
    }

    fn previous(&mut self) {
        match &self.state {
            AppState::Menu => {
                self.menu.previous();
            }
            AppState::ViewingOutput(_) | AppState::Error(_) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }

    fn page_down(&mut self, amount: u16) {
        if self.is_in_subview() {
            self.scroll_offset = self.scroll_offset.saturating_add(amount);
        }
    }

    fn page_up(&mut self, amount: u16) {
        if self.is_in_subview() {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        }
    }

    fn should_refresh_view(&self) -> bool {
        if let AppState::ViewingOutput(view) = &self.state {
            if let Some(interval) = view.auto_refresh_interval() {
                if let Some(last_refresh) = self.last_refresh {
                    return last_refresh.elapsed().as_secs_f32() >= interval;
                }
                // If we've never refreshed, we should refresh now
                return true;
            }
        }
        false
    }

    fn refresh_current_view(&mut self, controller: &Controller) {
        if let AppState::ViewingOutput(view) = &mut self.state {
            // Preserve scroll position during auto-refresh
            let _ = view.refresh_data(controller);
            self.last_refresh = Some(Instant::now());
        }
    }

    fn manual_refresh(&mut self, controller: &Controller) {
        if let AppState::ViewingOutput(view) = &mut self.state {
            // Reset scroll position on manual refresh
            let _ = view.refresh_data(controller);
            self.last_refresh = Some(Instant::now());
            self.scroll_offset = 0;
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

            let ctrl = Controller {
                manager,
                lookup: &lookup,
                util: &util,
            };

            loop {
                let summoner_name = manager.get_summoner().game_name.clone();

                // Check if panic occurred and update state
                if let Ok(panic_guard) = self.panic_flag.lock() {
                    if let Some(panic_msg) = panic_guard.as_ref() {
                        self.state = AppState::Error(panic_msg.clone());
                    }
                }

                // Check if we should auto-refresh the current view
                if self.should_refresh_view() {
                    self.refresh_current_view(&ctrl);
                }

                let mut view_height = 0; // Placeholder initialization
                terminal.draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
                        .split(f.size());
                    view_height = chunks[1].height;

                    // Title with subtle welcome message
                    let title = Paragraph::new(format!(" Welcome, {}!", summoner_name))
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Rgb(200, 150, 0)))
                                .title("Sedidata - LoL Special Statistics")
                                .title_style(
                                    Style::default()
                                        .fg(Color::Rgb(200, 150, 0))
                                        .add_modifier(Modifier::BOLD),
                                ),
                        );
                    f.render_widget(title, chunks[0]);

                    let info = match &self.state {
                        AppState::Menu => {
                            let store_status = if manager.get_store_responses() {
                                "ON"
                            } else {
                                "OFF"
                            };
                            format!("Use ↑/↓ to navigate, Enter to select, r to refresh data, s to toggle response storage [{}], q to quit.", store_status)
                        }
                        AppState::ViewingOutput(_) => {
                            "Use ↑/↓ or PgUp/PgDown to scroll, Esc/q to return.".to_string()
                        }
                        AppState::Error(_) => {
                            "Press 'q' to quit.".to_string()
                        }
                    };
                    let info_paragraph = Paragraph::new(info)
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Right);
                    f.render_widget(info_paragraph, chunks[2]);

                    // Render current state
                    match &mut self.state {
                        AppState::Error(panic_msg) => {
                            // Render panic error
                            let error_block = Block::default()
                                .borders(Borders::ALL)
                                .title("ERROR - Application Panicked")
                                .title_style(
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                )
                                .padding(ratatui::widgets::Padding::horizontal(1))
                                .border_style(Style::default().fg(Color::Red));

                            let error_text = Paragraph::new(panic_msg.as_str())
                                .block(error_block)
                                .wrap(Wrap { trim: false })
                                .scroll((self.scroll_offset, 0))
                                .style(Style::default().fg(Color::Red));

                            f.render_widget(error_text, chunks[1]);
                        }
                        AppState::Menu => self.menu.render(f, chunks[1]),
                        AppState::ViewingOutput(view) => {
                            // Always update view (polls async data)
                            view.update(&ctrl, &self.pressed_keys);
                            self.pressed_keys.clear();

                            // Render the view
                            let block = Block::default()
                                .borders(ratatui::widgets::Borders::ALL)
                                .padding(ratatui::widgets::Padding::horizontal(1))
                                .title(view.title().to_string())
                                .title_style(
                                    Style::default()
                                        .fg(Color::Rgb(200, 150, 0))
                                        .add_modifier(ratatui::style::Modifier::BOLD),
                                )
                                .border_style(Style::default().fg(Color::Rgb(200, 150, 0)));

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
                            KeyCode::Char('q') if !self.is_in_subview() => {
                                self.should_quit = true;
                                break;
                            }
                            KeyCode::Char('s') if self.is_in_menu() => {
                                manager.toggle_store_responses();
                            }
                            KeyCode::Char('r') if self.is_in_menu() => {
                                self.should_refresh = true;
                                break;
                            }
                            KeyCode::Char('r') if self.is_in_subview() => {
                                // Manual refresh in view mode
                                let ctrl = Controller {
                                    manager,
                                    lookup: &lookup,
                                    util: &util,
                                };
                                self.manual_refresh(&ctrl);
                            }
                            KeyCode::Up => self.previous(),
                            KeyCode::Down => self.next(),
                            KeyCode::PageUp => self.page_up(view_height / 2),
                            KeyCode::PageDown => self.page_down(view_height / 2),
                            KeyCode::Esc | KeyCode::Char('q') if self.is_in_subview() => {
                                self.state = AppState::Menu;
                                self.scroll_offset = 0;
                                self.last_refresh = None;
                            }
                            KeyCode::Enter if self.is_in_menu() => {
                                if let Some(factory) = self.menu.get_factory() {
                                    let view = factory(&ctrl);

                                    terminal.clear()?;
                                    self.state = AppState::ViewingOutput(view);
                                    self.scroll_offset = 0;
                                    self.last_refresh = Some(Instant::now());
                                }
                            }
                            _ => {
                                self.pressed_keys.push(key.code);
                            }
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
        let champions = manager.get_champions().recv().unwrap()?;
        let skins = manager.get_skins().recv().unwrap()?;
        let masteries = manager.get_masteries().recv().unwrap()?;
        let challenges = manager.get_challenges().recv().unwrap()?;
        let queues = manager.get_queue_types().recv().unwrap()?;

        Ok(LookupService::new(&champions, &skins, &masteries, &challenges, &queues))
    }
}

pub fn run(mut manager: DataManager) -> Result<(), ReplError> {
    // Enable backtrace in debug builds
    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Create panic flag
    let panic_flag = Arc::new(Mutex::new(None));
    let panic_flag_hook = panic_flag.clone();

    // Set up panic hook to store panic info
    std::panic::set_hook(Box::new(move |panic_info| {
        let mut msg = String::from("Application panicked!\n\n");

        // Location
        if let Some(location) = panic_info.location() {
            msg.push_str(&format!(
                "Location: {}:{}:{}\n\n",
                location.file(),
                location.line(),
                location.column()
            ));
        }

        // Message
        msg.push_str("Message:\n");
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            msg.push_str(&format!("  {}\n\n", s));
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            msg.push_str(&format!("  {}\n\n", s));
        } else {
            msg.push_str("  <no message>\n\n");
        }

        // Thread info
        if let Some(thread_name) = std::thread::current().name() {
            msg.push_str(&format!("Thread: {}\n\n", thread_name));
        } else {
            msg.push_str(&format!(
                "Thread: <unnamed> (id: {:?})\n\n",
                std::thread::current().id()
            ));
        }

        // Backtrace
        msg.push_str("Backtrace:\n");

        // Check environment variable
        let backtrace_enabled = std::env::var("RUST_BACKTRACE")
            .map(|v| v == "1" || v.to_lowercase() == "full")
            .unwrap_or(false);
        msg.push_str(&format!(
            "  RUST_BACKTRACE={:?}\n",
            std::env::var("RUST_BACKTRACE").ok()
        ));

        if backtrace_enabled {
            let backtrace = std::backtrace::Backtrace::force_capture();
            msg.push_str(&format!("{}\n", backtrace));
        } else {
            msg.push_str("  <disabled - run with RUST_BACKTRACE=1 to enable>\n");
        }

        if let Ok(mut panic_info_guard) = panic_flag_hook.lock() {
            *panic_info_guard = Some(msg);
        }
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(panic_flag);
    let result = app.run(&mut terminal, &mut manager);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = &result {
        eprintln!("Error: {}", err);
    }

    result
}
