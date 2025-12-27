use super::ids::{ChampionId, ChromaId, SkinId};

#[derive(Debug)]
pub struct AllChampionInfo {
    pub champions: Vec<Champion>,
    pub skins: Vec<Skin>,
    pub chromas: Vec<Chroma>,
}

#[derive(Debug, Clone)]
pub struct Champion {
    pub id: ChampionId,
    pub name: String,
    pub active: bool,
    pub owned: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub id: SkinId,
    pub champ_id: ChampionId,
    pub name: String,
    pub is_base: bool,
    pub owned: bool,
}

#[derive(Debug, Clone)]
pub struct Chroma {
    pub id: ChromaId,
    pub skin_id: SkinId,
    pub owned: bool,
}

impl Champion {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.to_lowercase() == role)
    }
}
