use super::ids::ChampionId;

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
    pub _puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub is_ally: bool,
    pub selected_champion: ChampionId,
}

#[derive(Debug)]
pub struct QueueInfo {
    pub queue_id: u16,
    pub _category: String,
    pub _description: String,
    pub _gamemode: String,
    pub _type_descriptor: String,
    pub select_mode_group: String,
}
