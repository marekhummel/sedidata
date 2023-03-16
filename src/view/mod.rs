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
    InitFailed(DataRetrievalError),
    ConsoleFailed(io::Error),
}

impl From<DataRetrievalError> for ReplError {
    fn from(error: DataRetrievalError) -> Self {
        ReplError::InitFailed(error)
    }
}

impl From<io::Error> for ReplError {
    fn from(error: io::Error) -> Self {
        ReplError::ConsoleFailed(error)
    }
}
