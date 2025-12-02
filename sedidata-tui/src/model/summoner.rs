use std::collections::HashMap;

use super::ids::SummonerId;

#[derive(Debug, Clone)]
pub struct Summoner {
    pub id: SummonerId,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub level: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct RiotApiSummonerResponse {
    pub level: u16,
    pub ranked_stats: Vec<RankedQueueStats>,
    pub champion_mastery_info: Option<(u16, u32)>,
}

#[derive(Debug, Clone)]
pub struct SummonerWithStats {
    pub summoner: Summoner,
    pub ranked_stats: Option<HashMap<String, RankedQueueStats>>,
    pub champion_mastery: Option<(u16, u32)>,
}

#[derive(Debug, Clone)]
pub struct RankedQueueStats {
    pub queue_type: String,
    pub tier: String,
    pub division: String,
    pub league_points: u32,
    pub wins: u32,
    pub losses: u32,
}
