use super::ids::SummonerId;

#[derive(Debug, Clone)]
pub struct Summoner {
    pub id: SummonerId,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub level: u16,
}

#[derive(Debug, Clone)]
pub struct RankedQueueStats {
    pub queue_type: String,
    pub tier: String,
    pub division: String,
    pub league_points: u32,
    pub wins: u32,
    pub _losses: u32,
    pub _is_provisional: bool,
    pub highest_tier: String,
    pub highest_division: String,
}
