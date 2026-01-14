use std::collections::HashMap;

use super::ids::SummonerId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SummonerName {
    pub game_name: String,
    pub tag_line: String,
}

impl SummonerName {
    pub fn full(&self) -> String {
        format!("{}#{}", self.game_name, self.tag_line)
    }

    pub fn tuple(&self) -> (String, String) {
        (self.game_name.clone(), self.tag_line.clone())
    }
}

impl Default for SummonerName {
    fn default() -> Self {
        SummonerName {
            game_name: "".into(),
            tag_line: "".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Summoner {
    pub id: SummonerId,
    pub puuid: String,
    pub name: SummonerName,
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
    pub champion_mastery: PlayedChampionMasteryInfo,
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

#[derive(Debug, Clone)]
pub struct PlayedChampionMasteryInfo {
    pub champion_name: Option<String>,
    pub level_points: Option<(u16, u32)>,
}
