use core::fmt;

pub mod challenge;
pub mod champion;
pub mod champselect;
pub mod livegame;
pub mod loot;
pub mod mastery;
pub mod queues;
pub mod summoner;

#[derive(Debug)]
pub enum ParsingError {
    InvalidType(String),
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParsingError::InvalidType(field) => write!(f, "Invalid type for field: {}", field),
        }
    }
}
