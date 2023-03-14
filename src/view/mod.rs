use crate::service::{data_manager::DataRetrievalError, dictionary::DictionaryError};

mod inventory;
pub mod repl;

#[derive(Debug)]
pub enum ViewError {
    ManagerFailed(DataRetrievalError),
    DictionaryFailed(DictionaryError),
}

impl From<DataRetrievalError> for ViewError {
    fn from(error: DataRetrievalError) -> Self {
        ViewError::ManagerFailed(error)
    }
}

impl From<DictionaryError> for ViewError {
    fn from(error: DictionaryError) -> Self {
        ViewError::DictionaryFailed(error)
    }
}
