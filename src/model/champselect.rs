use super::ids::ChampionId;

#[derive(Debug)]
pub struct ChampSelectInfo {
    pub current_champ_id: ChampionId,
    pub team_champs: Vec<SelectableChamp>,
    pub benched_champs: Vec<SelectableChamp>,
}

#[derive(Debug)]
pub struct SelectableChamp {
    pub id: ChampionId,
    pub selectable: bool,
}
