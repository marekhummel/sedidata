use std::{
    fmt,
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use json::JsonValue;
use reqwest::blocking::Client;

use crate::model::{champion::Champion, ids::ChampionId, summoner::SummonerName};

const BASE_URL: &str = "https://sedidata-server.onrender.com";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5 * 60); // 5 minutes

pub struct RiotApiClient {
    client: Client,
}

impl RiotApiClient {
    pub fn new() -> Result<Self, RiotApiClientInitError> {
        let client = Client::builder().timeout(Duration::from_secs(90)).build()?;

        // Clone for heartbeat thread
        let heartbeat_client = client.clone();

        // Spawn heartbeat thread
        thread::spawn(move || {
            Self::heartbeat_loop(heartbeat_client);
        });

        Ok(Self { client })
    }

    fn heartbeat_loop(client: Client) {
        loop {
            let url = format!("{}/heartbeat", BASE_URL);
            let _is_alive = match client.get(&url).send() {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            };

            thread::sleep(HEARTBEAT_INTERVAL);
        }
    }

    pub fn get_multiple_player_info(
        &self,
        players: &[(Option<SummonerName>, Option<Champion>)],
    ) -> Vec<(Option<SummonerName>, RiotApiClientResult<Arc<JsonValue>>)> {
        let (tx, rx) = mpsc::channel();

        // Spawn a thread for each request
        for (name, champ) in players {
            let client = self.client.clone();
            let tx = tx.clone();
            let name = name.clone();
            let champ = champ.clone();

            thread::spawn(move || {
                let result = Self::fetch_player_info(&client, &name, &champ.as_ref().map(|c| c.id.clone()));
                let _ = tx.send((name, result));
            });
        }

        // Drop the original sender so rx knows when all threads are done
        drop(tx);

        // Collect all results
        rx.into_iter().collect()
    }

    fn fetch_player_info(
        client: &Client,
        name: &Option<SummonerName>,
        champ: &Option<ChampionId>,
    ) -> RiotApiClientResult<Arc<JsonValue>> {
        let Some(summ_name) = name else {
            return Ok(Arc::new(JsonValue::Null));
        };

        let mut url = format!(
            "{}/league?name={}&tagline={}",
            BASE_URL,
            urlencoding::encode(&summ_name.game_name),
            urlencoding::encode(&summ_name.tag_line)
        );

        if let Some(champ_id) = champ {
            url.push_str(&format!("&champion={}", champ_id.0));
        }

        let response = client.get(&url).send()?;

        if !response.status().is_success() {
            return Err(RiotApiRequestError::InvalidResponse(
                response.status().as_u16(),
                response.text().unwrap_or_else(|_| "Unknown error".to_string()),
            ));
        }

        let text = response.text()?;
        let json = json::parse(&text)?;

        Ok(Arc::new(json))
    }
}

#[derive(Debug)]
pub enum RiotApiClientInitError {
    HttpClientCreation(reqwest::Error),
}

impl fmt::Display for RiotApiClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RiotApiClientInitError::HttpClientCreation(e) => {
                write!(f, "Failed to create HTTP client: {}", e)
            }
        }
    }
}

impl From<reqwest::Error> for RiotApiClientInitError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpClientCreation(error)
    }
}

pub type RiotApiClientResult<T> = Result<T, RiotApiRequestError>;

#[derive(Debug)]
pub enum RiotApiRequestError {
    NetworkError(reqwest::Error),
    InvalidResponse(u16, String),
    JsonParseError(json::Error),
}

impl fmt::Display for RiotApiRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RiotApiRequestError::NetworkError(e) => {
                write!(f, "Network error: {}", e)
            }
            RiotApiRequestError::InvalidResponse(status, body) => {
                write!(f, "Server returned error {}: {}", status, body)
            }
            RiotApiRequestError::JsonParseError(e) => {
                write!(f, "Failed to parse JSON response: {}", e)
            }
        }
    }
}

impl From<reqwest::Error> for RiotApiRequestError {
    fn from(error: reqwest::Error) -> Self {
        Self::NetworkError(error)
    }
}

impl From<json::Error> for RiotApiRequestError {
    fn from(error: json::Error) -> Self {
        Self::JsonParseError(error)
    }
}
