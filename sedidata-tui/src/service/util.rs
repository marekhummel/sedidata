use std::collections::HashSet;

use crate::{
    model::{
        champion::{Champion, Chroma, Skin},
        ids::{ChampionId, SkinId},
        mastery::Mastery,
    },
    service::data_manager::{DataManager, DataRetrievalResult},
};
use std::sync::mpsc::Receiver;
pub struct UtilService<'a> {
    manager: &'a DataManager,
}

impl<'a> UtilService<'a> {
    pub fn new(manager: &'a DataManager) -> Self {
        log::debug!("Creating UtilService");
        Self { manager }
    }

    pub fn get_owned_champions(&self) -> Receiver<DataRetrievalResult<Vec<Champion>>> {
        log::debug!("Fetching owned champions");
        let rx = self.manager.get_champions();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|champs| champs.into_iter().filter(|c| c.owned).collect())
        })
    }

    pub fn get_played_champions_set(&self) -> Receiver<DataRetrievalResult<HashSet<ChampionId>>> {
        log::debug!("Fetching played champions set");
        let rx = self.manager.get_masteries();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|masteries| masteries.iter().map(|m| m.champ_id.clone()).collect::<HashSet<_>>())
        })
    }

    pub fn get_champions_sorted_by_mastery(
        &self,
        maxpts: Option<u32>,
        minpts: Option<u32>,
    ) -> Receiver<DataRetrievalResult<Vec<ChampionId>>> {
        log::debug!(
            "Fetching champions sorted by mastery with maxpts={:?}, minpts={:?}",
            maxpts,
            minpts
        );
        let rx = self.manager.get_masteries();
        self.manager.async_wrapper(move || {
            rx.recv().unwrap().map(|masteries| {
                let mut sorted = masteries.clone();
                sorted.sort_by_key(|m| m.points);
                sorted.reverse();

                sorted
                    .iter()
                    .filter(|m| maxpts.unwrap_or(u32::MAX) >= m.points && m.points >= minpts.unwrap_or(0))
                    .map(|m| m.champ_id.clone())
                    .collect()
            })
        })
    }

    pub fn get_owned_skins(&self) -> Receiver<DataRetrievalResult<Vec<Skin>>> {
        log::debug!("Fetching owned skins");
        let rx = self.manager.get_skins();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|skins| skins.into_iter().filter(|s| s.owned).collect())
        })
    }

    pub fn get_owned_nobase_skins(&self) -> Receiver<DataRetrievalResult<Vec<Skin>>> {
        log::debug!("Fetching owned non-base skins");
        let rx = self.manager.get_skins();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|skins| skins.into_iter().filter(|s| s.owned && !s.is_base).collect())
        })
    }

    pub fn get_owned_chromas(&self) -> Receiver<DataRetrievalResult<Vec<Chroma>>> {
        log::debug!("Fetching owned chromas");
        let rx = self.manager.get_chromas();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|chromas| chromas.into_iter().filter(|s| s.owned).collect())
        })
    }

    pub fn get_owned_skins_set(&self) -> Receiver<DataRetrievalResult<HashSet<SkinId>>> {
        log::debug!("Fetching owned skins set");
        let rx = self.get_owned_skins();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|owned_skins| owned_skins.iter().map(|s| s.id.clone()).collect())
        })
    }

    pub fn get_masteries_with_level(&self, levels: Vec<u16>) -> Receiver<DataRetrievalResult<Vec<Mastery>>> {
        log::debug!("Fetching masteries with levels {:?}", levels);
        let rx = self.manager.get_masteries();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|masteries| masteries.into_iter().filter(|c| levels.contains(&c.level)).collect())
        })
    }

    pub fn _get_champ_shard_set(&self) -> Receiver<DataRetrievalResult<HashSet<ChampionId>>> {
        log::debug!("Fetching champion shard set");
        let rx = self.manager.get_loot();
        self.manager.async_wrapper(move || {
            rx.recv()
                .unwrap()
                .map(|loot| loot.champion_shards.iter().map(|cs| cs.champ_id.clone()).collect())
        })
    }
}
