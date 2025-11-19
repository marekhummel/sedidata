use crate::service::data_manager::DataRetrievalError;
use std::sync::mpsc::{Receiver, TryRecvError};

pub enum DataState<T> {
    Loading,
    Loaded(T),
    Error(String),
}

pub struct AsyncData<T> {
    state: DataState<T>,
    receiver: Option<Receiver<Result<T, DataRetrievalError>>>,
}

impl<T> AsyncData<T> {
    pub fn new(receiver: Receiver<Result<T, DataRetrievalError>>) -> Self {
        Self {
            state: DataState::Loading,
            receiver: Some(receiver),
        }
    }

    pub fn try_update(&mut self) {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(result) => {
                    self.state = match result {
                        Ok(data) => DataState::Loaded(data),
                        Err(e) => DataState::Error(format!("{}", e)),
                    };
                    self.receiver = None; // Done receiving
                }
                Err(TryRecvError::Empty) => {
                    // Still loading, do nothing
                }
                Err(TryRecvError::Disconnected) => {
                    // Sender dropped without sending
                    self.state = DataState::Error("Data fetch failed: channel disconnected".to_string());
                    self.receiver = None;
                }
            }
        }
    }

    pub fn get_data(&self) -> Option<&T> {
        match &self.state {
            DataState::Loaded(data) => Some(data),
            _ => None,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.state, DataState::Loading)
    }

    pub fn error(&self) -> Option<&str> {
        match &self.state {
            DataState::Error(e) => Some(e),
            _ => None,
        }
    }
}
