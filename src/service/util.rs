use std::collections::HashSet;

use crate::model::{
    champion::{Champion, Chroma, Skin},
    ids::{ChampionId, SkinId},
    mastery::Mastery,
};

use super::data_manager::{DataManager, DataRetrievalResult};

pub struct UtilService<'a> {
    manager: &'a DataManager,
}

impl<'a> UtilService<'a> {
    pub fn new(manager: &'a DataManager) -> Self {
        Self { manager }
    }

    pub fn get_owned_champions(&self) -> DataRetrievalResult<Vec<&Champion>> {
        let champs = self.manager.get_champions()?;
        Ok(champs.iter().filter(|c| c.owned).collect())
    }

    pub fn get_played_champions_set(&self) -> DataRetrievalResult<HashSet<ChampionId>> {
        Ok(self
            .manager
            .get_masteries()?
            .iter()
            .map(|m| m.champ_id.clone())
            .collect::<HashSet<_>>())
    }

    pub fn get_champions_sorted_by_mastery(
        &self,
        maxpts: Option<u32>,
        minpts: Option<u32>,
    ) -> DataRetrievalResult<Vec<ChampionId>> {
        let mut masteries = self.manager.get_masteries()?.iter().collect::<Vec<_>>();
        masteries.sort_by_key(|m| m.points);
        masteries.reverse();

        Ok(masteries
            .iter()
            .filter(|m| maxpts.unwrap_or(u32::MAX) >= m.points && m.points >= minpts.unwrap_or(0))
            .map(|m| m.champ_id.clone())
            .collect())
    }

    pub fn get_owned_skins(&self) -> DataRetrievalResult<Vec<&Skin>> {
        let skins = self.manager.get_skins()?;
        Ok(skins.iter().filter(|s| s.owned).collect())
    }

    pub fn get_owned_nobase_skins(&self) -> DataRetrievalResult<Vec<&Skin>> {
        let skins = self.manager.get_skins()?;
        Ok(skins.iter().filter(|s| s.owned && !s.is_base).collect())
    }

    pub fn get_owned_chromas(&self) -> DataRetrievalResult<Vec<&Chroma>> {
        let chromas = self.manager.get_chromas()?;
        Ok(chromas.iter().filter(|s| s.owned).collect())
    }

    pub fn get_owned_skins_set(&self) -> DataRetrievalResult<HashSet<SkinId>> {
        let owned_skins = self.get_owned_skins()?;
        Ok(owned_skins.iter().map(|s| s.id.clone()).collect())
    }

    pub fn get_masteries_with_level(&self, levels: Vec<u16>) -> DataRetrievalResult<Vec<&Mastery>> {
        let masteries = self.manager.get_masteries()?;
        Ok(masteries.iter().filter(|c| levels.contains(&c.level)).collect())
    }

    pub fn _get_champ_shard_set(&self) -> DataRetrievalResult<HashSet<ChampionId>> {
        let loot = self.manager.get_loot()?;
        let masteries = &loot.champion_shards;

        Ok(masteries.iter().map(|cs| cs.champ_id.clone()).collect())
    }
}
