use std::collections::HashMap;

use crate::model::{
    champion::{Champion, Skin},
    ids::{ChampionId, SkinId},
    mastery::Mastery,
};

pub struct LookupService<'a> {
    champs: HashMap<ChampionId, &'a Champion>,
    skins: HashMap<SkinId, &'a Skin>,
    masteries: HashMap<ChampionId, &'a Mastery>,
}

impl<'a> LookupService<'a> {
    pub fn new(
        champions: &'a Vec<Champion>,
        skins: &'a Vec<Skin>,
        masteries: &'a Vec<Mastery>,
    ) -> Self {
        Self {
            champs: champions.iter().map(|c| (c.id.clone(), c)).collect(),
            skins: skins.iter().map(|c| (c.id.clone(), c)).collect(),
            masteries: masteries.iter().map(|m| (m.champ_id.clone(), m)).collect(),
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

    pub fn get_mastery(&self, id: &ChampionId) -> Result<&'a Mastery, LookupError> {
        match self.masteries.get(id) {
            Some(mastery) => Ok(*mastery),
            None => Err(LookupError::ChampIdNotFound(id.clone())),
        }
    }
}

#[derive(Debug)]
pub enum LookupError {
    ChampIdNotFound(ChampionId),
    SkinIdNotFound(SkinId),
}
