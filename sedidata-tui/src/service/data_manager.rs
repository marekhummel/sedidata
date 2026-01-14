use std::{
    collections::HashMap,
    fmt,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

use itertools::Itertools;

use crate::{
    model::{
        challenge::Challenge,
        champion::{AllChampionInfo, Champion, Chroma, Skin},
        game::{ChampSelectSession, LiveGameSession, PostGameSession, QueueInfo},
        loot::LootItems,
        mastery::Mastery,
        summoner::{PlayedChampionMasteryInfo, Summoner, SummonerName, SummonerWithStats},
    },
    service::gameapi::{
        lcu_client::{LcuClient, LcuClientInitError, LcuClientRequestType, LcuRequestError},
        live_game_client::{LiveGameClient, LiveGameRequestError},
        parsing::{
            challenge::parse_challenges,
            champion::parse_champions,
            champselect::parse_champ_select,
            livegame::parse_live_game,
            loot::parse_loot,
            mastery::parse_masteries,
            postgame::parse_post_game,
            queues::parse_queues,
            summoner::{parse_ranked_stats, parse_summoner},
            ParsingError,
        },
        riot_api_client::{RiotApiClient, RiotApiClientInitError, RiotApiRequestError},
    },
};
pub struct DataManager {
    lcu_client: Arc<LcuClient>,
    live_game_client: Arc<LiveGameClient>,
    riot_api_client: Arc<RiotApiClient>,
    summoner: Arc<Mutex<Option<Summoner>>>,
    champ_info_cache: Arc<Mutex<Option<AllChampionInfo>>>,
    masteries_cache: Arc<Mutex<Option<Vec<Mastery>>>>,
    loot_cache: Arc<Mutex<Option<LootItems>>>,
    challenges_cache: Arc<Mutex<Option<Vec<Challenge>>>>,
    queues_cache: Arc<Mutex<Option<Vec<QueueInfo>>>>,
    store_responses: Arc<Mutex<bool>>,
}

impl DataManager {
    pub fn new(load_local: bool) -> Result<Self, DataManagerInitError> {
        let store_responses = Arc::new(Mutex::new(false));
        let mut client = LcuClient::new(load_local, Arc::clone(&store_responses))?;
        let live_game_client = LiveGameClient::new(load_local, Arc::clone(&store_responses));
        let riot_api_client = RiotApiClient::new()?;
        let summoner = DataManager::retrieve_summoner(&mut client)?;
        client.set_summoner(summoner.clone());

        Ok(Self {
            lcu_client: Arc::new(client),
            live_game_client: Arc::new(live_game_client),
            riot_api_client: Arc::new(riot_api_client),
            summoner: Arc::new(Mutex::new(Some(summoner))),
            champ_info_cache: Arc::new(Mutex::new(None)),
            masteries_cache: Arc::new(Mutex::new(None)),
            loot_cache: Arc::new(Mutex::new(None)),
            challenges_cache: Arc::new(Mutex::new(None)),
            queues_cache: Arc::new(Mutex::new(None)),
            store_responses,
        })
    }

    pub fn get_store_responses(&self) -> bool {
        *self.store_responses.lock().unwrap()
    }

    pub fn toggle_store_responses(&self) {
        let mut flag = self.store_responses.lock().unwrap();
        *flag = !*flag;
    }

    // Generic async wrapper that executes fetch in a thread
    pub fn async_wrapper<T, F>(&self, fetch_fn: F) -> Receiver<DataRetrievalResult<T>>
    where
        T: Send + 'static,
        F: FnOnce() -> DataRetrievalResult<T> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = fetch_fn();
            tx.send(result).ok();
        });

        rx
    }

    pub fn get_summoner(&self) -> Summoner {
        self.summoner.lock().unwrap().clone().unwrap()
    }

    pub fn get_champions(&self) -> Receiver<DataRetrievalResult<Vec<Champion>>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.champ_info_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(champ_info) = cache_guard.as_ref() {
                return Ok(champ_info.champions.clone());
            }

            let champs_json = client.request(LcuClientRequestType::Champions, true)?;
            let champ_info = parse_champions(Arc::as_ref(&champs_json))?;
            let result = champ_info.champions.clone();

            *cache_guard = Some(champ_info);
            Ok(result)
        })
    }

    pub fn get_skins(&self) -> Receiver<DataRetrievalResult<Vec<Skin>>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.champ_info_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(champ_info) = cache_guard.as_ref() {
                return Ok(champ_info.skins.clone());
            }

            let champs_json = client.request(LcuClientRequestType::Champions, true)?;
            let champ_info = parse_champions(Arc::as_ref(&champs_json))?;
            let result = champ_info.skins.clone();

            *cache_guard = Some(champ_info);
            Ok(result)
        })
    }

    pub fn get_chromas(&self) -> Receiver<DataRetrievalResult<Vec<Chroma>>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.champ_info_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(champ_info) = cache_guard.as_ref() {
                return Ok(champ_info.chromas.clone());
            }

            let champs_json = client.request(LcuClientRequestType::Champions, true)?;
            let champ_info = parse_champions(Arc::as_ref(&champs_json))?;
            let result = champ_info.chromas.clone();

            *cache_guard = Some(champ_info);
            Ok(result)
        })
    }

    pub fn get_masteries(&self) -> Receiver<DataRetrievalResult<Vec<Mastery>>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.masteries_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(masteries) = cache_guard.as_ref() {
                return Ok(masteries.clone());
            }

            let masteries_json = client.request(LcuClientRequestType::Masteries, true)?;
            let masteries = parse_masteries(Arc::as_ref(&masteries_json))?;

            *cache_guard = Some(masteries.clone());
            Ok(masteries)
        })
    }

    pub fn get_loot(&self) -> Receiver<DataRetrievalResult<LootItems>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.loot_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(loot) = cache_guard.as_ref() {
                return Ok(loot.clone());
            }

            let loot_json = client.request(LcuClientRequestType::Loot, true)?;
            let loot = parse_loot(Arc::as_ref(&loot_json))?;

            *cache_guard = Some(loot.clone());
            Ok(loot)
        })
    }

    pub fn get_challenges(&self) -> Receiver<DataRetrievalResult<Vec<Challenge>>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.challenges_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(challenges) = cache_guard.as_ref() {
                return Ok(challenges.clone());
            }

            let challenges_json = client.request(LcuClientRequestType::Challenges, true)?;
            let challenges = parse_challenges(Arc::as_ref(&challenges_json))?;

            *cache_guard = Some(challenges.clone());
            Ok(challenges)
        })
    }

    pub fn get_queue_types(&self) -> Receiver<DataRetrievalResult<Vec<QueueInfo>>> {
        let client = Arc::clone(&self.lcu_client);
        let cache = Arc::clone(&self.queues_cache);

        self.async_wrapper(move || {
            let mut cache_guard = cache.lock().unwrap();

            if let Some(queues) = cache_guard.as_ref() {
                return Ok(queues.clone());
            }

            let queues_json = client.request(LcuClientRequestType::QueueTypes, true)?;
            let queues = parse_queues(Arc::as_ref(&queues_json))?;

            *cache_guard = Some(queues.clone());
            Ok(queues)
        })
    }

    pub fn get_champ_select(&self) -> Receiver<DataRetrievalResult<Option<ChampSelectSession>>> {
        let client = Arc::clone(&self.lcu_client);

        self.async_wrapper(move || match client.request(LcuClientRequestType::ChampSelect, false) {
            Ok(champ_select_json) => {
                let champ_select_info = parse_champ_select(Arc::as_ref(&champ_select_json))?;
                Ok(Some(champ_select_info))
            }
            Err(LcuRequestError::InvalidResponse(_, _)) => Ok(None),
            Err(LcuRequestError::LocalFileError(_)) => Ok(None),
            Err(err) => Err(err.into()),
        })
    }

    pub fn get_live_game(&self) -> Receiver<DataRetrievalResult<Option<LiveGameSession>>> {
        let client = Arc::clone(&self.live_game_client);

        self.async_wrapper(move || match client.request() {
            Ok(live_game_json) => {
                let live_game_info = parse_live_game(&live_game_json)?;
                Ok(Some(live_game_info))
            }
            Err(LiveGameRequestError::LocalFileError(_)) => Ok(None),
            Err(err) => Err(err.into()),
        })
    }

    pub fn get_post_game(&self) -> Receiver<DataRetrievalResult<Option<PostGameSession>>> {
        let client = Arc::clone(&self.lcu_client);

        self.async_wrapper(move || match client.request(LcuClientRequestType::EndOfGame, false) {
            Ok(post_game_json) => {
                let post_game_info = parse_post_game(Arc::as_ref(&post_game_json))?;
                Ok(Some(post_game_info))
            }
            Err(LcuRequestError::InvalidResponse(_, _)) => Ok(None),
            Err(LcuRequestError::LocalFileError(_)) => Ok(None),
            Err(err) => Err(err.into()),
        })
    }

    pub fn get_ranked_info(
        &self,
        players: Vec<(Option<SummonerName>, Option<Champion>)>,
    ) -> Receiver<DataRetrievalResult<Vec<SummonerWithStats>>> {
        let riot_client = Arc::clone(&self.riot_api_client);

        self.async_wrapper(move || {
            // Fetch and update cache
            let riot_api_response = riot_client.get_multiple_player_info(&players.iter().cloned().collect_vec());

            let mut results = Vec::new();
            for (name, response_json) in riot_api_response {
                if let Ok(json) = &response_json {
                    if let Ok(parsed) = parse_ranked_stats(json.as_ref()) {
                        results.push((name, Some(parsed)));
                        continue;
                    }
                }
                results.push((name, None));
            }

            let champion_name_lookup: HashMap<_, _> = players
                .iter()
                .filter_map(|(on, oc)| on.clone().zip(oc.clone()).map(|(n, c)| (n, c.name)))
                .collect();

            // Map to SummonerWithStats and return
            Ok(results
                .into_iter()
                .filter_map(|(name, resp)| {
                    let summ_name = name?;
                    let summoner = Summoner {
                        id: 0.into(),
                        puuid: "".into(),
                        name: summ_name.clone(),
                        level: resp.clone().map(|r| r.level),
                    };
                    let champion_name = champion_name_lookup.get(&summ_name).cloned();
                    Some(SummonerWithStats {
                        summoner,
                        ranked_stats: resp.as_ref().map(|r| {
                            r.ranked_stats
                                .iter()
                                .map(|r| (r.queue_type.clone(), r.clone()))
                                .collect()
                        }),
                        champion_mastery: PlayedChampionMasteryInfo {
                            champion_name,
                            level_points: resp.as_ref().and_then(|r| r.champion_mastery_info),
                        },
                    })
                })
                .collect_vec())
        })
    }

    pub fn refresh(&mut self) -> DataRetrievalResult<()> {
        // Get mutable reference to lcu_client (need to deref Arc)
        let client = Arc::get_mut(&mut self.lcu_client).ok_or(DataRetrievalError::ClientRefresh(
            LcuClientInitError::LocalAppDataNotFound,
        ))?;

        client.refresh()?;
        let summoner = DataManager::retrieve_summoner(client)?;
        client.set_summoner(summoner.clone());

        *self.summoner.lock().unwrap() = Some(summoner);
        *self.champ_info_cache.lock().unwrap() = None;
        *self.masteries_cache.lock().unwrap() = None;
        *self.loot_cache.lock().unwrap() = None;
        *self.challenges_cache.lock().unwrap() = None;
        *self.queues_cache.lock().unwrap() = None;

        Ok(())
    }

    fn retrieve_summoner(client: &mut LcuClient) -> DataRetrievalResult<Summoner> {
        let summoner_json = client.request(LcuClientRequestType::Summoner, true)?;
        let summoner = parse_summoner(Arc::as_ref(&summoner_json))?;
        Ok(summoner)
    }
}

pub type DataRetrievalResult<T> = Result<T, DataRetrievalError>;

#[derive(Debug)]
pub enum DataManagerInitError {
    LcuClientFailed(LcuClientInitError),
    RiotApiClientFailed(RiotApiClientInitError),
    SummonerNotFound(DataRetrievalError),
}

impl fmt::Display for DataManagerInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataManagerInitError::LcuClientFailed(err) => write!(f, "Client initialization error: {}", err),
            DataManagerInitError::RiotApiClientFailed(err) => {
                write!(f, "Riot API client initialization error: {}", err)
            }
            DataManagerInitError::SummonerNotFound(err) => write!(f, "Summoner retrieval error: {}", err),
        }
    }
}

impl From<LcuClientInitError> for DataManagerInitError {
    fn from(error: LcuClientInitError) -> Self {
        Self::LcuClientFailed(error)
    }
}

impl From<RiotApiClientInitError> for DataManagerInitError {
    fn from(error: RiotApiClientInitError) -> Self {
        Self::RiotApiClientFailed(error)
    }
}

impl From<DataRetrievalError> for DataManagerInitError {
    fn from(error: DataRetrievalError) -> Self {
        Self::SummonerNotFound(error)
    }
}

#[derive(Debug)]
pub enum DataRetrievalError {
    LcuClient(LcuRequestError),
    LiveGameClient(LiveGameRequestError),
    RiotApiClient(RiotApiRequestError),
    ClientRefresh(LcuClientInitError),
    Parsing(ParsingError),
}

impl fmt::Display for DataRetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataRetrievalError::LcuClient(err) => write!(f, "Client error: {}", err),
            DataRetrievalError::LiveGameClient(err) => write!(f, "Live game client error: {}", err),
            DataRetrievalError::RiotApiClient(err) => write!(f, "Riot API client error: {}", err),
            DataRetrievalError::ClientRefresh(err) => write!(f, "Refresh error: {}", err),
            DataRetrievalError::Parsing(err) => write!(f, "Parsing error: {}", err),
        }
    }
}

impl From<LcuRequestError> for DataRetrievalError {
    fn from(error: LcuRequestError) -> Self {
        Self::LcuClient(error)
    }
}

impl From<LiveGameRequestError> for DataRetrievalError {
    fn from(error: LiveGameRequestError) -> Self {
        Self::LiveGameClient(error)
    }
}

impl From<RiotApiRequestError> for DataRetrievalError {
    fn from(error: RiotApiRequestError) -> Self {
        Self::RiotApiClient(error)
    }
}

impl From<LcuClientInitError> for DataRetrievalError {
    fn from(error: LcuClientInitError) -> Self {
        Self::ClientRefresh(error)
    }
}

impl From<ParsingError> for DataRetrievalError {
    fn from(error: ParsingError) -> Self {
        Self::Parsing(error)
    }
}
