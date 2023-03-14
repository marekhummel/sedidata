use std::collections::HashMap;

use crate::model::{
    champion::{Champion, Skin},
    ids::{ChampionId, SkinId},
};

pub struct Dictionary<'a> {
    champs: HashMap<ChampionId, &'a Champion>,
    skins: HashMap<SkinId, &'a Skin>,
}

impl<'a> Dictionary<'a> {
    pub fn new(champions: &'a Vec<Champion>, skins: &'a Vec<Skin>) -> Self {
        Self {
            champs: champions.iter().map(|c| (c.id.clone(), c)).collect(),
            skins: skins.iter().map(|c| (c.id.clone(), c)).collect(),
        }
    }

    pub fn get_champion(&self, id: ChampionId) -> Result<&'a Champion, DictionaryError> {
        match self.champs.get(&id) {
            Some(champ) => Ok(*champ),
            None => Err(DictionaryError::ChampIdNotFound(id)),
        }
    }

    pub fn get_skin(&self, id: SkinId) -> Result<&'a Skin, DictionaryError> {
        match self.skins.get(&id) {
            Some(skin) => Ok(*skin),
            None => Err(DictionaryError::SkinIdNotFound(id)),
        }
    }
}

#[derive(Debug)]
pub enum DictionaryError {
    ChampIdNotFound(ChampionId),
    SkinIdNotFound(SkinId),
}
