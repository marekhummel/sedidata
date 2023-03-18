use chrono::{DateTime, Utc};

use super::ids::ChampionId;

#[derive(Debug)]
pub struct Game {
    pub champ_id: ChampionId,
    pub queue: QueueType,
    pub season: u8,
    pub timestamp: DateTime<Utc>,
    pub stats: Statistics,
}

#[derive(Debug)]
pub struct Statistics {
    pub kills: u16,
    pub deaths: u16,
    pub assists: u16,
    pub doubles: u16,
    pub triples: u16,
    pub quadras: u16,
    pub pentas: u16,
}

#[derive(Debug)]
pub enum QueueType {
    Blind,
    Draft,
    RankedSolo,
    RankedFlex,
}
