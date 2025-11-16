use std::{fmt, rc::Rc};

use once_cell::sync::OnceCell;

use crate::{
    model::{
        challenge::Challenge,
        champion::{AllChampionInfo, Champion, Chroma, Skin},
        champselect::{ChampSelectPlayer, ChampSelectSession, QueueInfo},
        loot::LootItems,
        mastery::Mastery,
        summoner::Summoner,
    },
    service::gameapi::{
        client::{ApiClient, ClientInitError, ClientRequestType, RequestError},
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
    },
};
pub struct DataManager {
    client: ApiClient,
    summoner: OnceCell<Summoner>,
    champ_info_cache: OnceCell<AllChampionInfo>,
    masteries_cache: OnceCell<Vec<Mastery>>,
    loot_cache: OnceCell<LootItems>,
    challenges_cache: OnceCell<Vec<Challenge>>,
    queues_cache: OnceCell<Vec<QueueInfo>>,
}

impl DataManager {
    pub fn new(load_local: bool, write_responses: bool) -> Result<Self, DataManagerInitError> {
        let mut client = ApiClient::new(load_local, write_responses)?;
        let summoner = DataManager::retrieve_summoner(&mut client)?;
        client.set_summoner(summoner.clone());

        Ok(Self {
            client,
            summoner: OnceCell::from(summoner),
            champ_info_cache: OnceCell::new(),
            masteries_cache: OnceCell::new(),
            loot_cache: OnceCell::new(),
            challenges_cache: OnceCell::new(),
            queues_cache: OnceCell::new(),
        })
    }

    pub fn get_summoner(&self) -> &Summoner {
        self.summoner.get().unwrap()
    }

    pub fn get_champions(&self) -> DataRetrievalResult<&Vec<Champion>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions, true)?;
                let champ_info = parse_champions(Rc::as_ref(&champs_json))?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.champions)
    }

    pub fn get_skins(&self) -> DataRetrievalResult<&Vec<Skin>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions, true)?;
                let champ_info = parse_champions(Rc::as_ref(&champs_json))?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.skins)
    }
    pub fn get_chromas(&self) -> DataRetrievalResult<&Vec<Chroma>> {
        self.champ_info_cache
            .get_or_try_init(|| {
                let champs_json = self.client.request(ClientRequestType::Champions, true)?;
                let champ_info = parse_champions(Rc::as_ref(&champs_json))?;
                Ok(champ_info)
            })
            .map(|champ_info| &champ_info.chromas)
    }

    pub fn get_masteries(&self) -> DataRetrievalResult<&Vec<Mastery>> {
        self.masteries_cache.get_or_try_init(|| {
            let masteries_json = self.client.request(ClientRequestType::Masteries, true)?;
            let masteries = parse_masteries(Rc::as_ref(&masteries_json))?;
            Ok(masteries)
        })
    }

    pub fn get_loot(&self) -> DataRetrievalResult<&LootItems> {
        self.loot_cache.get_or_try_init(|| {
            let loot_json = self.client.request(ClientRequestType::Loot, true)?;
            let loot = parse_loot(Rc::as_ref(&loot_json))?;
            Ok(loot)
        })
    }

    pub fn get_challenges(&self) -> DataRetrievalResult<&Vec<Challenge>> {
        self.challenges_cache.get_or_try_init(|| {
            let challenges_json = self.client.request(ClientRequestType::Challenges, true)?;
            let challenges = parse_challenges(Rc::as_ref(&challenges_json))?;
            Ok(challenges)
        })
    }

    pub fn get_queue_types(&self) -> DataRetrievalResult<&Vec<QueueInfo>> {
        self.queues_cache.get_or_try_init(|| {
            let queues_json = self.client.request(ClientRequestType::QueueTypes, true)?;
            let queues = parse_queues(Rc::as_ref(&queues_json))?;
            Ok(queues)
        })
    }

    pub fn get_champ_select(&self) -> DataRetrievalResult<Option<ChampSelectSession>> {
        match self.client.request(ClientRequestType::ChampSelect, false) {
            Ok(champ_select_json) => {
                let champ_select_info = parse_champ_select(Rc::as_ref(&champ_select_json))?;
                Ok(Some(champ_select_info))
            }
            Err(RequestError::InvalidResponse(_, _)) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn get_champ_select_with_ranked(
        &self,
    ) -> DataRetrievalResult<Option<(ChampSelectSession, Vec<ChampSelectPlayer>)>> {
        let sessio_opt = self.get_champ_select()?;
        match sessio_opt {
            Some(session) => {
                let mut all_players = Vec::new();

                // Process all players from both teams
                for player_info in session.my_team.iter().chain(session.their_team.iter()) {
                    // Fetch summoner info
                    let summoner_json = self
                        .client
                        .request(ClientRequestType::OtherSummoner(player_info.puuid.clone()), false)?;

                    let summoner = parse_summoner(Rc::as_ref(&summoner_json))?;

                    // Fetch ranked stats
                    let ranked_json = self
                        .client
                        .request(ClientRequestType::RankedStats(player_info.puuid.clone()), false)?;

                    let ranked_stats = parse_ranked_stats(Rc::as_ref(&ranked_json))?;

                    all_players.push(ChampSelectPlayer {
                        player_info: player_info.clone(),
                        summoner: Some(summoner),
                        ranked_stats,
                    });
                }

                Ok(Some((session, all_players)))
            }
            None => Ok(None),
        }
    }

    pub fn refresh(&mut self) -> DataRetrievalResult<()> {
        self.client.refresh()?;
        let summoner = DataManager::retrieve_summoner(&mut self.client)?;
        self.client.set_summoner(summoner.clone());
        self.summoner = OnceCell::from(summoner);
        self.champ_info_cache = OnceCell::new();
        self.masteries_cache = OnceCell::new();
        self.loot_cache = OnceCell::new();
        self.challenges_cache = OnceCell::new();
        self.queues_cache = OnceCell::new();
        Ok(())
    }

    fn retrieve_summoner(client: &mut ApiClient) -> DataRetrievalResult<Summoner> {
        let summoner_json = client.request(ClientRequestType::Summoner, true)?;
        let summoner = parse_summoner(Rc::as_ref(&summoner_json))?;
        Ok(summoner)
    }
}

pub type DataRetrievalResult<T> = Result<T, DataRetrievalError>;

#[derive(Debug)]
pub enum DataManagerInitError {
    ClientFailed(ClientInitError),
    SummonerNotFound(DataRetrievalError),
}

impl fmt::Display for DataManagerInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataManagerInitError::ClientFailed(err) => write!(f, "Client initialization error: {}", err),
            DataManagerInitError::SummonerNotFound(err) => write!(f, "Summoner retrieval error: {}", err),
        }
    }
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
    Client(RequestError),
    ClientRefresh(ClientInitError),
    Parsing(ParsingError),
}

impl fmt::Display for DataRetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataRetrievalError::Client(err) => write!(f, "Client error: {}", err),
            DataRetrievalError::ClientRefresh(err) => write!(f, "Refresh error: {}", err),
            DataRetrievalError::Parsing(err) => write!(f, "Parsing error: {}", err),
        }
    }
}

impl From<RequestError> for DataRetrievalError {
    fn from(error: RequestError) -> Self {
        Self::Client(error)
    }
}

impl From<ClientInitError> for DataRetrievalError {
    fn from(error: ClientInitError) -> Self {
        Self::ClientRefresh(error)
    }
}

impl From<ParsingError> for DataRetrievalError {
    fn from(error: ParsingError) -> Self {
        Self::Parsing(error)
    }
}
