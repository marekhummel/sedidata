use std::io;

use crate::service::{data_manager::DataRetrievalError, lookup::LookupError};

pub mod repl;
mod subviews;

type ViewResult = Result<(), ViewError>;

#[derive(Debug)]
pub enum ViewError {
    ManagerFailed(DataRetrievalError),
    LookupFailed(LookupError),
}

impl From<DataRetrievalError> for ViewError {
    fn from(error: DataRetrievalError) -> Self {
        ViewError::ManagerFailed(error)
    }
}

impl From<LookupError> for ViewError {
    fn from(error: LookupError) -> Self {
        ViewError::LookupFailed(error)
    }
}

#[derive(Debug)]
pub enum ReplError {
    Init(DataRetrievalError),
    View(ViewError),
    Console(io::Error),
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
