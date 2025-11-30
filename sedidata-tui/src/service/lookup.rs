use std::{collections::HashMap, fmt};

use crate::model::{
    challenge::Challenge,
    champion::{Champion, Skin},
    game::QueueInfo,
    ids::{ChampionId, SkinId},
    mastery::Mastery,
};

pub struct LookupService {
    champs: HashMap<ChampionId, Champion>,
    skins: HashMap<SkinId, Skin>,
    masteries: HashMap<ChampionId, Mastery>,
    _challenges: HashMap<i32, Challenge>,
    queues: HashMap<u16, QueueInfo>,
}

impl LookupService {
    pub fn new(
        champions: &[Champion],
        skins: &[Skin],
        masteries: &[Mastery],
        challenges: &[Challenge],
        queues: &[QueueInfo],
    ) -> Self {
        Self {
            champs: champions.iter().map(|c| (c.id.clone(), c.clone())).collect(),
            skins: skins.iter().map(|c| (c.id.clone(), c.clone())).collect(),
            masteries: masteries.iter().map(|m| (m.champ_id.clone(), m.clone())).collect(),
            _challenges: challenges.iter().map(|ch| (ch.id, ch.clone())).collect(),
            queues: queues.iter().map(|q| (q.queue_id, q.clone())).collect(),
        }
    }

    pub fn get_champion(&self, id: &ChampionId) -> Result<Champion, IdNotFoundError> {
        match self.champs.get(id) {
            Some(champ) => Ok(champ.clone()),
            None => Err(IdNotFoundError::Champ(id.clone())),
        }
    }

    pub fn get_skin(&self, id: &SkinId) -> Result<Skin, IdNotFoundError> {
        match self.skins.get(id) {
            Some(skin) => Ok(skin.clone()),
            None => Err(IdNotFoundError::Skin(id.clone())),
        }
    }

    pub fn get_mastery(&self, id: &ChampionId) -> Result<Mastery, IdNotFoundError> {
        match self.masteries.get(id) {
            Some(mastery) => Ok(mastery.clone()),
            None => Err(IdNotFoundError::Champ(id.clone())),
        }
    }

    pub fn _get_challenge(&self, id: i32) -> Result<Challenge, IdNotFoundError> {
        match self._challenges.get(&id) {
            Some(challenge) => Ok(challenge.clone()),
            None => Err(IdNotFoundError::_Challenge(id)),
        }
    }

    pub fn get_queue(&self, id: u16) -> Result<QueueInfo, IdNotFoundError> {
        match self.queues.get(&id) {
            Some(queue) => Ok(queue.clone()),
            None => Err(IdNotFoundError::Queue(id)),
        }
    }
}

#[derive(Debug)]
pub enum IdNotFoundError {
    Champ(ChampionId),
    Skin(SkinId),
    _Challenge(i32),
    Queue(u16),
}

impl fmt::Display for IdNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdNotFoundError::Champ(id) => write!(f, "Champion ID not found: {}", id),
            IdNotFoundError::Skin(id) => write!(f, "Skin ID not found: {}", id),
            IdNotFoundError::_Challenge(id) => write!(f, "Challenge ID not found: {}", id),
            IdNotFoundError::Queue(id) => write!(f, "Queue ID not found: {}", id),
        }
    }
}
