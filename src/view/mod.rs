use std::{fmt, io};

use crate::service::{data_manager::DataRetrievalError, lookup::IdNotFoundError};

pub mod repl;
mod subviews;

type ViewResult = Result<(), ViewError>;

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
