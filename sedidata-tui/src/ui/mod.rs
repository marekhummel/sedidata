use std::{fmt, io};

use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::{layout::Rect, text::Line, Frame};

use crate::service::{data_manager::DataManager, lookup::LookupService, util::UtilService};
use crate::service::{data_manager::DataRetrievalError, lookup::IdNotFoundError};

pub mod async_data;
pub mod menu;
pub mod repl;
pub mod views;

pub use async_data::AsyncData;

pub type TextCreationResult = Result<Vec<Line<'static>>, ViewError>;
type ViewResult = Result<(), ViewError>;

pub struct Controller<'a> {
    pub manager: &'a DataManager,
    pub lookup: &'a LookupService,
    pub util: &'a UtilService<'a>,
}

pub struct RenderContext<'a, 'b> {
    pub frame: &'a mut Frame<'b>,
    pub area: Rect,
    pub scroll_offset: u16,
    pub block: Block<'b>,
}

impl<'a, 'b> RenderContext<'a, 'b> {
    pub fn error(self, error: &str) {
        let paragraph = Paragraph::new(format!("\n  [!] Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true })
            .block(self.block)
            .scroll((0, 0));

        self.frame.render_widget(paragraph, self.area);
    }
}

#[derive(Debug)]
pub enum ViewError {
    ManagerFailed(DataRetrievalError),
    LookupFailed(IdNotFoundError),
}

impl fmt::Display for ViewError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ViewError::ManagerFailed(err) => write!(f, "Data manager error: {}", err),
            ViewError::LookupFailed(err) => write!(f, "Lookup service error: {}", err),
        }
    }
}

impl From<DataRetrievalError> for ViewError {
    fn from(error: DataRetrievalError) -> Self {
        ViewError::ManagerFailed(error)
    }
}

impl From<IdNotFoundError> for ViewError {
    fn from(error: IdNotFoundError) -> Self {
        ViewError::LookupFailed(error)
    }
}

#[derive(Debug)]
pub enum ReplError {
    Init(DataRetrievalError),
    View(ViewError),
    Console(io::Error),
}

impl fmt::Display for ReplError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplError::Init(err) => write!(f, "Init error: {}", err),
            ReplError::View(err) => write!(f, "View error: {}", err),
            ReplError::Console(err) => write!(f, "Console error: {}", err),
        }
    }
}

impl From<DataRetrievalError> for ReplError {
    fn from(error: DataRetrievalError) -> Self {
        ReplError::Init(error)
    }
}

impl From<ViewError> for ReplError {
    fn from(error: ViewError) -> Self {
        ReplError::View(error)
    }
}

impl From<io::Error> for ReplError {
    fn from(error: io::Error) -> Self {
        ReplError::Console(error)
    }
}
