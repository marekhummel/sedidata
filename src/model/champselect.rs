use super::ids::ChampionId;

use crate::model::summoner::{RankedQueueStats, Summoner};

#[derive(Debug, Clone)]
pub struct ChampSelectSession {
    pub queue_id: u16,
    pub local_player_cell: u8,
    pub benched_champs: Vec<ChampionId>,
    pub my_team: Vec<ChampSelectPlayerInfo>,
    pub their_team: Vec<ChampSelectPlayerInfo>,
}

#[derive(Debug, Clone)]
pub struct ChampSelectPlayerInfo {
    pub cell_id: u8,
    pub position: String,
    pub puuid: String,
    pub is_ally: bool,
    pub selected_champion: ChampionId,
}

#[derive(Debug, Clone)]
pub struct ChampSelectPlayer {
    pub player_info: ChampSelectPlayerInfo,
    pub summoner: Option<Summoner>,
    pub ranked_stats: Vec<RankedQueueStats>,
}

#[derive(Debug)]
pub struct QueueInfo {
    pub queue_id: u16,
    pub _category: String,
    pub _description: String,
    pub gamemode: String,
    pub _type_descriptor: String,
}
