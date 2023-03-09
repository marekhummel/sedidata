use json::JsonValue;

use crate::{
    gameapi::{
        client::{ClientInitError, RequestError},
        parsing::{champion::parse_champions, loot::parse_loot, summoner::parse_summoner},
        parsing::{mastery::parse_masteries, ParsingError},
    },
    model::{
        cache_wrapper::CacheWrapper,
        champion::{AllChampionInfo, Champion, Chroma, Skin},
        loot::{JsonLootItem, LootItems},
        mastery::Mastery,
        summoner::Summoner,
    },
};

use crate::gameapi::client::{ApiClient, ClientRequestType};

pub struct DataManager {
    client: ApiClient,
    summoner: Summoner,
    champ_info_cache: CacheWrapper<AllChampionInfo>,
    masteries_cache: CacheWrapper<Vec<Mastery>>,
    loot_cache: CacheWrapper<LootItems>,
}

impl DataManager {
    pub fn new(league_path: &str) -> Result<Self, DataManagerInitError> {
        let mut client = ApiClient::new(league_path)?;
        let summoner = DataManager::retrieve_summoner(&mut client)?;
        client.set_summoner_id(summoner.id.clone());

        Ok(Self {
            client,
            summoner,
            champ_info_cache: CacheWrapper::new(),
            masteries_cache: CacheWrapper::new(),
            loot_cache: CacheWrapper::new(),
        })
    }

    pub fn get_summoner(&self) -> &Summoner {
        &self.summoner
    }

    pub fn get_champions(&mut self) -> Result<&Vec<Champion>, DataRetrievalError> {
        if self.champ_info_cache.is_empty() {
            let champs_json = self.client.request(ClientRequestType::Champions)?;
            let champ_info = parse_champions(champs_json)?;
            self.champ_info_cache.set(champ_info);
        }

        Ok(&self.champ_info_cache.content().champions)
    }

    pub fn get_skins(&mut self) -> Result<&Vec<Skin>, DataRetrievalError> {
        if self.champ_info_cache.is_empty() {
            let champs_json = self.client.request(ClientRequestType::Champions)?;
            let champ_info = parse_champions(champs_json)?;
            self.champ_info_cache.set(champ_info);
        }

        Ok(&self.champ_info_cache.content().skins)
    }
    pub fn get_chromas(&mut self) -> Result<&Vec<Chroma>, DataRetrievalError> {
        if self.champ_info_cache.is_empty() {
            let champs_json = self.client.request(ClientRequestType::Champions)?;
            let champ_info = parse_champions(champs_json)?;
            self.champ_info_cache.set(champ_info);
        }

        Ok(&self.champ_info_cache.content().chromas)
    }

    pub fn get_masteries(&mut self) -> Result<&Vec<Mastery>, DataRetrievalError> {
        if self.masteries_cache.is_empty() {
            let masteries_json = self.client.request(ClientRequestType::Masteries)?;
            let masteries = parse_masteries(masteries_json)?;
            self.masteries_cache.set(masteries);
        }

        Ok(&self.masteries_cache.content())
    }

    pub fn get_loot(&mut self) -> Result<&LootItems, DataRetrievalError> {
        if self.loot_cache.is_empty() {
            let loot_json = self.client.request(ClientRequestType::Loot)?;
            let loot = parse_loot(loot_json)?;
            self.loot_cache.set(loot);
        }

        Ok(&self.loot_cache.content())
    }

    pub fn refresh(&mut self) -> Result<(), DataRetrievalError> {
        self.client.refresh();
        let summoner = DataManager::retrieve_summoner(&mut self.client)?;
        self.client.set_summoner_id(summoner.id.clone());
        self.summoner = summoner;
        self.champ_info_cache = CacheWrapper::new();
        self.masteries_cache = CacheWrapper::new();
        self.loot_cache = CacheWrapper::new();
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
