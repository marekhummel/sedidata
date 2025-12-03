use std::{
    fmt,
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use json::JsonValue;
use reqwest::blocking::Client;

use crate::model::{champion::Champion, ids::ChampionId};

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
        players: &[(String, String, Option<Champion>)],
    ) -> Vec<(String, String, Result<Arc<JsonValue>, RiotApiRequestError>)> {
        let (tx, rx) = mpsc::channel();

        // Spawn a thread for each request
        for (name, tagline, champ) in players {
            let client = self.client.clone();
            let tx = tx.clone();
            let name = name.clone();
            let tagline = tagline.clone();
            let champ = champ.clone();

            thread::spawn(move || {
                let result = Self::fetch_player_info(&client, &name, &tagline, &champ.as_ref().map(|c| c.id.clone()));
                let _ = tx.send((name, tagline, result));
            });
        }

        // Drop the original sender so rx knows when all threads are done
        drop(tx);

        // Collect all results
        rx.into_iter().collect()
    }

    fn fetch_player_info(
        client: &Client,
        name: &str,
        tagline: &str,
        champ: &Option<ChampionId>,
    ) -> Result<Arc<JsonValue>, RiotApiRequestError> {
        if name.is_empty() || tagline.is_empty() {
            return Ok(Arc::new(JsonValue::Null));
        }

        let mut url = format!(
            "{}/league?name={}&tagline={}",
            BASE_URL,
            urlencoding::encode(name),
            urlencoding::encode(tagline)
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
