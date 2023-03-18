use std::collections::HashMap;

use crate::model::{
    champion::{Champion, Skin},
    ids::{ChampionId, SkinId},
};

pub struct LookupService<'a> {
    champs: HashMap<ChampionId, &'a Champion>,
    skins: HashMap<SkinId, &'a Skin>,
}

impl<'a> LookupService<'a> {
    pub fn new(champions: &'a Vec<Champion>, skins: &'a Vec<Skin>) -> Self {
        Self {
            champs: champions.iter().map(|c| (c.id.clone(), c)).collect(),
            skins: skins.iter().map(|c| (c.id.clone(), c)).collect(),
        }
    }

    pub fn get_champion(&self, id: &ChampionId) -> Result<&'a Champion, LookupError> {
        match self.champs.get(id) {
            Some(champ) => Ok(*champ),
            None => Err(LookupError::ChampIdNotFound(id.clone())),
        }
    }

    pub fn get_skin(&self, id: &SkinId) -> Result<&'a Skin, LookupError> {
        match self.skins.get(id) {
            Some(skin) => Ok(*skin),
            None => Err(LookupError::SkinIdNotFound(id.clone())),
        }
    }
}

#[derive(Debug)]
pub enum LookupError {
    ChampIdNotFound(ChampionId),
    SkinIdNotFound(SkinId),
}
