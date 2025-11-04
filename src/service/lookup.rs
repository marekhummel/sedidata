use std::{collections::HashMap, fmt};

use crate::model::{
    challenge::Challenge,
    champion::{Champion, Skin},
    ids::{ChampionId, SkinId},
    mastery::Mastery,
};

pub struct LookupService<'a> {
    champs: HashMap<ChampionId, &'a Champion>,
    skins: HashMap<SkinId, &'a Skin>,
    masteries: HashMap<ChampionId, &'a Mastery>,
    _challenges: HashMap<i32, &'a Challenge>,
}

impl<'a> LookupService<'a> {
    pub fn new(
        champions: &'a [Champion],
        skins: &'a [Skin],
        masteries: &'a [Mastery],
        challenges: &'a [Challenge],
    ) -> Self {
        Self {
            champs: champions.iter().map(|c| (c.id.clone(), c)).collect(),
            skins: skins.iter().map(|c| (c.id.clone(), c)).collect(),
            masteries: masteries.iter().map(|m| (m.champ_id.clone(), m)).collect(),
            _challenges: challenges.iter().map(|ch| (ch.id, ch)).collect(),
        }
    }

    pub fn get_champion(&self, id: &ChampionId) -> Result<&'a Champion, IdNotFoundError> {
        match self.champs.get(id) {
            Some(champ) => Ok(*champ),
            None => Err(IdNotFoundError::Champ(id.clone())),
        }
    }

    pub fn get_skin(&self, id: &SkinId) -> Result<&'a Skin, IdNotFoundError> {
        match self.skins.get(id) {
            Some(skin) => Ok(*skin),
            None => Err(IdNotFoundError::Skin(id.clone())),
        }
    }

    pub fn get_mastery(&self, id: &ChampionId) -> Result<&'a Mastery, IdNotFoundError> {
        match self.masteries.get(id) {
            Some(mastery) => Ok(*mastery),
            None => Err(IdNotFoundError::Champ(id.clone())),
        }
    }

    pub fn _get_challenge(&self, id: i32) -> Result<&'a Challenge, IdNotFoundError> {
        match self._challenges.get(&id) {
            Some(challenge) => Ok(*challenge),
            None => Err(IdNotFoundError::_Challenge(id)),
        }
    }
}

#[derive(Debug)]
pub enum IdNotFoundError {
    Champ(ChampionId),
    Skin(SkinId),
    _Challenge(i32),
}

impl fmt::Display for IdNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdNotFoundError::Champ(id) => write!(f, "Champion ID not found: {}", id),
            IdNotFoundError::Skin(id) => write!(f, "Skin ID not found: {}", id),
            IdNotFoundError::_Challenge(id) => write!(f, "Challenge ID not found: {}", id),
        }
    }
}
