use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use itertools::Itertools;
use once_cell::sync::OnceCell;

use crate::{
    model::{
        challenge::Challenge,
        champion::{AllChampionInfo, Champion, Chroma, Skin},
        champselect::{ChampSelectSession, QueueInfo},
        loot::LootItems,
        mastery::Mastery,
        summoner::{RiotApiSummonerResponse, Summoner, SummonerWithStats},
    },
    service::gameapi::{
        lcu_client::{LcuClient, LcuClientInitError, LcuClientRequestType, LcuRequestError},
        parsing::{
            challenge::parse_challenges,
            champion::parse_champions,
            champselect::parse_champ_select,
            loot::parse_loot,
            mastery::parse_masteries,
            queues::parse_queues,
            summoner::{parse_ranked_stats, parse_summoner},
            ParsingError,
        },
        riot_api_client::{RiotApiClient, RiotApiClientInitError, RiotApiRequestError},
    },
};
pub struct DataManager {
    lcu_client: LcuClient,
    riot_api_client: RiotApiClient,
    summoner: OnceCell<Summoner>,
    champ_info_cache: OnceCell<AllChampionInfo>,
    masteries_cache: OnceCell<Vec<Mastery>>,
    loot_cache: OnceCell<LootItems>,
    challenges_cache: OnceCell<Vec<Challenge>>,
    queues_cache: OnceCell<Vec<QueueInfo>>,
    ranked_info_cache: RefCell<HashMap<(String, String), RiotApiSummonerResponse>>,
}

impl DataManager {
    pub fn new(load_local: bool, write_responses: bool) -> Result<Self, DataManagerInitError> {
        let mut client = LcuClient::new(load_local, write_responses)?;
        let riot_api_client = RiotApiClient::new()?;
        let summoner = DataManager::retrieve_summoner(&mut client)?;
        client.set_summoner(summoner.clone());

        Ok(Self {
            lcu_client: client,
            riot_api_client,
            summoner: OnceCell::from(summoner),
            champ_info_cache: OnceCell::new(),
            masteries_cache: OnceCell::new(),
            loot_cache: OnceCell::new(),
            challenges_cache: OnceCell::new(),
            queues_cache: OnceCell::new(),
            ranked_info_cache: RefCell::new(HashMap::new()),
        })
    }

    pub fn get_summoner(&self) -> &Summoner {
        self.summoner.get().unwrap()
    }

    pub fn get_champions(&self) -> DataRetrievalResult<&Vec<Champion>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.lcu_client.request(LcuClientRequestType::Champions, true)?;
                let champ_info = parse_champions(Rc::as_ref(&champs_json))?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.champions)
    }

    pub fn get_skins(&self) -> DataRetrievalResult<&Vec<Skin>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.lcu_client.request(LcuClientRequestType::Champions, true)?;
                let champ_info = parse_champions(Rc::as_ref(&champs_json))?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.skins)
    }
    pub fn get_chromas(&self) -> DataRetrievalResult<&Vec<Chroma>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.lcu_client.request(LcuClientRequestType::Champions, true)?;
                let champ_info = parse_champions(Rc::as_ref(&champs_json))?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.chromas)
    }

    pub fn get_masteries(&self) -> DataRetrievalResult<&Vec<Mastery>> {
        self.masteries_cache.get_or_try_init(|| {
            let masteries_json = self.lcu_client.request(LcuClientRequestType::Masteries, true)?;
            let masteries = parse_masteries(Rc::as_ref(&masteries_json))?;
            Ok(masteries)
        })
    }

    pub fn get_loot(&self) -> DataRetrievalResult<&LootItems> {
        self.loot_cache.get_or_try_init(|| {
            let loot_json = self.lcu_client.request(LcuClientRequestType::Loot, true)?;
            let loot = parse_loot(Rc::as_ref(&loot_json))?;
            Ok(loot)
        })
    }

    pub fn get_challenges(&self) -> DataRetrievalResult<&Vec<Challenge>> {
        self.challenges_cache.get_or_try_init(|| {
            let challenges_json = self.lcu_client.request(LcuClientRequestType::Challenges, true)?;
            let challenges = parse_challenges(Rc::as_ref(&challenges_json))?;
            Ok(challenges)
        })
    }

    pub fn get_queue_types(&self) -> DataRetrievalResult<&Vec<QueueInfo>> {
        self.queues_cache.get_or_try_init(|| {
            let queues_json = self.lcu_client.request(LcuClientRequestType::QueueTypes, true)?;
            let queues = parse_queues(Rc::as_ref(&queues_json))?;
            Ok(queues)
        })
    }

    pub fn get_champ_select(&self) -> DataRetrievalResult<Option<ChampSelectSession>> {
        match self.lcu_client.request(LcuClientRequestType::ChampSelect, false) {
            Ok(champ_select_json) => {
                let champ_select_info = parse_champ_select(Rc::as_ref(&champ_select_json))?;
                Ok(Some(champ_select_info))
            }
            Err(LcuRequestError::InvalidResponse(_, _)) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn get_ranked_info(&self, players: &[(String, String)]) -> DataRetrievalResult<Vec<SummonerWithStats>> {
        let mut cache = self.ranked_info_cache.borrow_mut();

        let (cached, fetch): (Vec<_>, Vec<_>) = players.iter().partition(|p| cache.contains_key(p));

        let cached_reponses = cached
            .into_iter()
            .map(|(name, tagline)| {
                (
                    name.clone(),
                    tagline.clone(),
                    Some(cache.get(&(name.clone(), tagline.clone())).unwrap().clone()),
                )
            })
            .collect_vec();

        // Fetch and update cache
        let riot_api_response = self
            .riot_api_client
            .get_multiple_player_info(&fetch.iter().map(|(n, t)| (n.clone(), t.clone())).collect_vec());

        let mut results = Vec::new();
        for (name, tagline, response_json) in riot_api_response {
            if let Ok(json) = &response_json {
                if let Ok(parsed) = parse_ranked_stats(json.as_ref()) {
                    cache.insert((name.clone(), tagline.clone()), parsed.clone());
                    results.push((name, tagline, Some(parsed)));
                    continue;
                }
            }
            results.push((name, tagline, None));
        }

        // Combine cached and fetched
        Ok(cached_reponses
            .into_iter()
            .chain(results)
            .map(|(name, tagline, resp)| {
                let summoner = Summoner {
                    id: 0.into(),
                    puuid: "".into(),
                    game_name: name.clone(),
                    tag_line: tagline.clone(),
                    level: resp.clone().map(|r| r.level).unwrap_or(0),
                };
                SummonerWithStats {
                    summoner,
                    ranked_stats: resp.map(|r| {
                        r.ranked_stats
                            .iter()
                            .map(|r| (r.queue_type.clone(), r.clone()))
                            .collect()
                    }),
                }
            })
            .collect_vec())
    }

    pub fn refresh(&mut self) -> DataRetrievalResult<()> {
        self.lcu_client.refresh()?;
        let summoner = DataManager::retrieve_summoner(&mut self.lcu_client)?;
        self.lcu_client.set_summoner(summoner.clone());
        self.summoner = OnceCell::from(summoner);
        self.champ_info_cache = OnceCell::new();
        self.masteries_cache = OnceCell::new();
        self.loot_cache = OnceCell::new();
        self.challenges_cache = OnceCell::new();
        self.queues_cache = OnceCell::new();
        Ok(())
    }

    fn retrieve_summoner(client: &mut LcuClient) -> DataRetrievalResult<Summoner> {
        let summoner_json = client.request(LcuClientRequestType::Summoner, true)?;
        let summoner = parse_summoner(Rc::as_ref(&summoner_json))?;
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
    RiotApiClient(RiotApiRequestError),
    ClientRefresh(LcuClientInitError),
    Parsing(ParsingError),
}

impl fmt::Display for DataRetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataRetrievalError::LcuClient(err) => write!(f, "Client error: {}", err),
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
