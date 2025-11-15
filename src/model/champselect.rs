use super::ids::ChampionId;

#[derive(Debug)]
pub struct ChampSelectInfo {
    pub current_champ_id: ChampionId,
    pub team_champs: Vec<ChampionId>,
    pub benched_champs: Vec<ChampionId>,
    pub queue_id: u16,
}

#[derive(Debug)]
pub struct QueueInfo {
    pub queue_id: u16,
    pub _category: String,
    pub _description: String,
    pub gamemode: String,
    pub _type_descriptor: String,
}
