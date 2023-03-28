use super::ids::ChampionId;

#[derive(Debug)]
pub struct ChampSelectInfo {
    pub current_champ_id: ChampionId,
    pub team_champs: Vec<ChampionId>,
    pub benched_champs: Vec<ChampionId>,
}
