use super::ids::ChampionId;

#[derive(Debug)]
pub struct Mastery {
    pub champ_id: ChampionId,
    pub level: u8,
    pub points: u32,
    pub tokens: Option<u8>,
    pub points_to_next_level: u32,
    pub chest_granted: bool,
}
