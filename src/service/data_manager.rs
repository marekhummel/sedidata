use once_cell::sync::OnceCell;

use crate::model::{
    champion::{AllChampionInfo, Champion, Chroma, Skin},
    loot::LootItems,
    mastery::Mastery,
    summoner::Summoner,
};

use super::gameapi::{
    client::{ApiClient, ClientInitError, ClientRequestType, RequestError},
    parsing::{
        champion::parse_champions, loot::parse_loot, mastery::parse_masteries,
        summoner::parse_summoner, ParsingError,
    },
};

pub struct DataManager {
    client: ApiClient,
    summoner: Summoner,
    champ_info_cache: OnceCell<AllChampionInfo>,
    masteries_cache: OnceCell<Vec<Mastery>>,
    loot_cache: OnceCell<LootItems>,
}

impl DataManager {
    pub fn new(league_path: &str) -> Result<Self, DataManagerInitError> {
        let mut client = ApiClient::new(league_path)?;
        let summoner = DataManager::retrieve_summoner(&mut client)?;
        client.set_summoner_id(summoner.id.clone());

        Ok(Self {
            client,
            summoner,
            champ_info_cache: OnceCell::new(),
            masteries_cache: OnceCell::new(),
            loot_cache: OnceCell::new(),
        })
    }

    pub fn get_summoner(&self) -> &Summoner {
        &self.summoner
    }

    pub fn get_champions(&self) -> Result<&Vec<Champion>, DataRetrievalError> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions)?;
                let champ_info = parse_champions(champs_json)?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.champions)
    }

    pub fn get_skins(&self) -> Result<&Vec<Skin>, DataRetrievalError> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions)?;
                let champ_info = parse_champions(champs_json)?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.skins)
    }
    pub fn get_chromas(&self) -> Result<&Vec<Chroma>, DataRetrievalError> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions)?;
                let champ_info = parse_champions(champs_json)?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.chromas)
    }

    pub fn get_masteries(&self) -> Result<&Vec<Mastery>, DataRetrievalError> {
        self.masteries_cache.get_or_try_init(|| {
            let masteries_json = self.client.request(ClientRequestType::Masteries)?;
            let masteries = parse_masteries(masteries_json)?;
            Ok(masteries)
        })
    }

    pub fn get_loot(&self) -> Result<&LootItems, DataRetrievalError> {
        self.loot_cache.get_or_try_init(|| {
            let loot_json = self.client.request(ClientRequestType::Loot)?;
            let loot = parse_loot(loot_json)?;
            Ok(loot)
        })
    }

    pub fn refresh(&mut self) -> Result<(), DataRetrievalError> {
        self.client.refresh();
        let summoner = DataManager::retrieve_summoner(&mut self.client)?;
        self.client.set_summoner_id(summoner.id.clone());
        self.summoner = summoner;
        self.champ_info_cache = OnceCell::new();
        self.masteries_cache = OnceCell::new();
        self.loot_cache = OnceCell::new();
        Ok(())
    }

    fn retrieve_summoner(client: &mut ApiClient) -> Result<Summoner, DataRetrievalError> {
        let summoner_json = client.request(ClientRequestType::Summoner)?;
        let summoner = parse_summoner(summoner_json)?;
        Ok(summoner)
    }
}

#[derive(Debug)]
pub enum DataManagerInitError {
    ClientFailed(ClientInitError),
    SummonerNotFound(DataRetrievalError),
}

impl From<ClientInitError> for DataManagerInitError {
    fn from(error: ClientInitError) -> Self {
        Self::ClientFailed(error)
    }
}

impl From<DataRetrievalError> for DataManagerInitError {
    fn from(error: DataRetrievalError) -> Self {
        Self::SummonerNotFound(error)
    }
}

#[derive(Debug)]
pub enum DataRetrievalError {
    ClientFailed(RequestError),
    ParsingFailed(ParsingError),
}

impl From<RequestError> for DataRetrievalError {
    fn from(error: RequestError) -> Self {
        Self::ClientFailed(error)
    }
}

impl From<ParsingError> for DataRetrievalError {
    fn from(error: ParsingError) -> Self {
        Self::ParsingFailed(error)
    }
}
