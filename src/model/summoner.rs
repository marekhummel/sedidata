use super::ids::SummonerId;

#[derive(Debug, Clone)]
pub struct Summoner {
    pub id: SummonerId,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub level: u16,
}
