use once_cell::sync::OnceCell;

use crate::model::{
    champion::{AllChampionInfo, Champion, Chroma, Skin},
    games::Game,
    loot::LootItems,
    mastery::Mastery,
    summoner::Summoner,
};

use super::gameapi::{
    client::{ApiClient, ClientInitError, ClientRequestType, RequestError},
    parsing::{
        champion::parse_champions, games::parse_game_stats, loot::parse_loot,
        mastery::parse_masteries, summoner::parse_summoner, ParsingError,
    },
};

pub struct DataManager {
    client: ApiClient,
    summoner: OnceCell<Summoner>,
    champ_info_cache: OnceCell<AllChampionInfo>,
    masteries_cache: OnceCell<Vec<Mastery>>,
    game_stats_cache: OnceCell<Vec<Game>>,
    loot_cache: OnceCell<LootItems>,
}

impl DataManager {
    pub fn new() -> Result<Self, DataManagerInitError> {
        let mut client = ApiClient::new()?;
        let summoner = DataManager::retrieve_summoner(&mut client)?;
        client.set_summoner(summoner.clone());

        Ok(Self {
            client,
            summoner: OnceCell::from(summoner),
            champ_info_cache: OnceCell::new(),
            masteries_cache: OnceCell::new(),
            game_stats_cache: OnceCell::new(),
            loot_cache: OnceCell::new(),
        })
    }

    pub fn get_summoner(&self) -> &Summoner {
        self.summoner.get().unwrap()
    }

    pub fn get_champions(&self) -> DataRetrievalResult<&Vec<Champion>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions)?;
                let champ_info = parse_champions(champs_json)?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.champions)
    }

    pub fn get_skins(&self) -> DataRetrievalResult<&Vec<Skin>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions)?;
                let champ_info = parse_champions(champs_json)?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.skins)
    }
    pub fn get_chromas(&self) -> DataRetrievalResult<&Vec<Chroma>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions)?;
                let champ_info = parse_champions(champs_json)?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.chromas)
    }

    pub fn get_masteries(&self) -> DataRetrievalResult<&Vec<Mastery>> {
        self.masteries_cache.get_or_try_init(|| {
            let masteries_json = self.client.request(ClientRequestType::Masteries)?;
            let masteries = parse_masteries(masteries_json)?;
            Ok(masteries)
        })
    }

    pub fn get_game_stats(&self) -> DataRetrievalResult<&Vec<Game>> {
        self.game_stats_cache.get_or_try_init(|| {
            let mut all_games = Vec::new();
            for season in 8..=13u8 {
                let games_json = self.client.request(ClientRequestType::GameStats(season))?;
                let games = parse_game_stats(games_json)?;
                all_games.extend(games);
            }
            Ok(all_games)
        })
    }

    pub fn get_loot(&self) -> DataRetrievalResult<&LootItems> {
        self.loot_cache.get_or_try_init(|| {
            let loot_json = self.client.request(ClientRequestType::Loot)?;
            let loot = parse_loot(loot_json)?;
            Ok(loot)
        })
    }

    pub fn refresh(&mut self) -> DataRetrievalResult<()> {
        self.client.refresh()?;
        let summoner = DataManager::retrieve_summoner(&mut self.client)?;
        self.client.set_summoner(summoner.clone());
        self.summoner = OnceCell::from(summoner);
        self.champ_info_cache = OnceCell::new();
        self.masteries_cache = OnceCell::new();
        self.loot_cache = OnceCell::new();
        Ok(())
    }

    fn retrieve_summoner(client: &mut ApiClient) -> DataRetrievalResult<Summoner> {
        let summoner_json = client.request(ClientRequestType::Summoner)?;
        let summoner = parse_summoner(summoner_json)?;
        Ok(summoner)
    }
}

pub type DataRetrievalResult<T> = Result<T, DataRetrievalError>;

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
    ClientRefreshFailed(ClientInitError),
    ParsingFailed(ParsingError),
}

impl From<RequestError> for DataRetrievalError {
    fn from(error: RequestError) -> Self {
        Self::ClientFailed(error)
    }
}

impl From<ClientInitError> for DataRetrievalError {
    fn from(error: ClientInitError) -> Self {
        Self::ClientRefreshFailed(error)
    }
}

impl From<ParsingError> for DataRetrievalError {
    fn from(error: ParsingError) -> Self {
        Self::ParsingFailed(error)
    }
}
