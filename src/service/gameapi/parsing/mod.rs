pub mod champion;
pub mod games;
pub mod loot;
pub mod mastery;
pub mod summoner;

#[derive(Debug)]
pub enum ParsingError {
    InvalidType(String),
}
