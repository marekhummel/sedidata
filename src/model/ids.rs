use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct SummonerId(String);

#[derive(Debug, Clone)]
pub struct ChampionId(String);

#[derive(Debug, Clone)]
pub struct SkinId(String);

#[derive(Debug, Clone)]
pub struct ChromaId(String);

impl Display for SummonerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for SummonerId {
    fn from(value: u64) -> Self {
        SummonerId(value.to_string())
    }
}

impl From<String> for ChampionId {
    fn from(value: String) -> Self {
        ChampionId(value)
    }
}

impl From<i32> for ChampionId {
    fn from(value: i32) -> Self {
        ChampionId(value.to_string())
    }
}

impl From<i32> for SkinId {
    fn from(value: i32) -> Self {
        SkinId(value.to_string())
    }
}

impl From<i32> for ChromaId {
    fn from(value: i32) -> Self {
        ChromaId(value.to_string())
    }
}
