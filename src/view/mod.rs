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
