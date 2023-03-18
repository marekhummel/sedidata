use super::ids::SummonerId;

#[derive(Debug, Clone)]
pub struct Summoner {
    pub id: SummonerId,
    pub puuid: String,
    pub display_name: String,
    pub internal_name: String,
    pub level: u16,
}
