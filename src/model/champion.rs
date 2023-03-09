use super::ids::{ChampionId, ChromaId, SkinId};

#[derive(Debug)]
pub struct AllChampionInfo {
    pub champions: Vec<Champion>,
    pub skins: Vec<Skin>,
    pub chromas: Vec<Chroma>,
}

#[derive(Debug)]
pub struct Champion {
    pub id: ChampionId,
    pub name: String,
    pub owned: bool,
}

#[derive(Debug)]
pub struct Skin {
    pub id: SkinId,
    pub champ: ChampionId,
    pub name: String,
    pub is_base: bool,
    pub owned: bool,
}

#[derive(Debug)]
pub struct Chroma {
    pub id: ChromaId,
    pub skin: SkinId,
    pub owned: bool,
}
