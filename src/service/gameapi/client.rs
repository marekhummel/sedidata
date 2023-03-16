use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, Read, Write},
    path::Path,
};

use base64::{engine::general_purpose, write::EncoderStringWriter};
use json::JsonValue;
use once_cell::sync::OnceCell;
use reqwest::{
    blocking::Client,
    header::{self, HeaderMap, HeaderValue, InvalidHeaderValue},
    Certificate,
};

use crate::model::ids::SummonerId;

#[derive(Debug)]
struct LockFileContent {
    base_url: String,
    username: String,
    password: String,
}

pub struct ApiClient {
    client: Client,
    cache: HashMap<ClientRequestType, OnceCell<JsonValue>>,
    base_url: String,
    summoner_id: Option<SummonerId>,
}

impl ApiClient {
    pub fn new(league_path: &str) -> Result<Self, ClientInitError> {
        // Read certificate
        let cert = ApiClient::read_certificate()?;

        // Read lockfile and create basic auth secret
        let lockfile = ApiClient::read_lockfile(league_path)?;
        let basic_auth = format!("{}:{}", lockfile.username, lockfile.password);
        let mut base64_enc = EncoderStringWriter::new(&general_purpose::STANDARD);
        base64_enc.write_all(basic_auth.as_bytes())?;
        let auth_secret = base64_enc.into_inner();

        // Create client with auth header
        let mut headers = HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(format!("Basic {}", auth_secret).as_str())?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        let client = Client::builder()
            .add_root_certificate(cert)
            .default_headers(headers)
            .build()?;

        // Create
        let cache = vec![
            (ClientRequestType::Summoner, OnceCell::new()),
            (ClientRequestType::Champions, OnceCell::new()),
            (ClientRequestType::Masteries, OnceCell::new()),
            (ClientRequestType::Loot, OnceCell::new()),
        ]
        .into_iter()
        .collect();
        Ok(Self {
            client,
            cache,
            base_url: lockfile.base_url,
            summoner_id: None,
        })
    }

    fn read_certificate() -> Result<Certificate, CertificateError> {
        let mut buffer = Vec::new();
        let mut cert_file = File::open("config/riotgames.pem")?;
        cert_file.read_to_end(&mut buffer)?;
        let cert = reqwest::Certificate::from_pem(&buffer)?;
        Ok(cert)
    }

    fn read_lockfile(league_path: &str) -> Result<LockFileContent, LockfileError> {
        let lol_path = Path::new(league_path);
        let lol_lockfile = File::open(lol_path.join("lockfile"))?;

        let content = io::BufReader::new(lol_lockfile)
            .lines()
            .next()
            .ok_or(LockfileError::CantBeRead)??;
        let info = content.split(":").collect::<Vec<_>>();

        Ok(LockFileContent {
            base_url: format!("{}://127.0.0.1:{}/", info[4], info[2]),
            username: "riot".to_string(),
            password: info[3].to_string(),
        })
    }

    pub fn request(&self, request_type: ClientRequestType) -> Result<&JsonValue, RequestError> {
        // Check for cache
        self.cache.get(&request_type).unwrap().get_or_try_init(|| {
            // Get url
            let url = match request_type {
                ClientRequestType::Summoner => {
                    format!("{}lol-summoner/v1/current-summoner", self.base_url)
                }
                ClientRequestType::Champions => match &self.summoner_id {
                    Some(sid) => format!(
                        "{}lol-champions/v1/inventories/{}/champions",
                        self.base_url, sid
                    ),
                    None => return Err(RequestError::SummonerIdNeeded()),
                },
                ClientRequestType::Masteries => match &self.summoner_id {
                    Some(sid) => format!(
                        "{}lol-collections/v1/inventories/{}/champion-mastery",
                        self.base_url, sid
                    ),
                    None => return Err(RequestError::SummonerIdNeeded()),
                },
                ClientRequestType::Loot => {
                    format!("{}lol-loot/v1/player-loot", self.base_url)
                }
            };

            // Send request
            let response = self.client.get(url).send()?;
            if !response.status().is_success() {
                return Err(RequestError::InvalidResponse());
            }

            // Return json
            let text = response.text()?;
            // let mut file = File::create(format!("data/{:?}.json", request_type)).unwrap();
            // let _ = file.write_all(text.as_bytes());
            let json = json::parse(text.as_str())?;
            Ok(json)
        })
    }

    pub fn set_summoner_id(&mut self, sid: SummonerId) {
        self.summoner_id = Some(sid);
    }

    pub fn refresh(&mut self) -> () {
        self.cache.clear();
        self.cache
            .insert(ClientRequestType::Summoner, OnceCell::new());
        self.cache
            .insert(ClientRequestType::Champions, OnceCell::new());
        self.cache
            .insert(ClientRequestType::Masteries, OnceCell::new());
        self.cache.insert(ClientRequestType::Loot, OnceCell::new());
        self.summoner_id = None;
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum ClientRequestType {
    Summoner,
    Champions,
    Masteries,
    Loot,
}

#[derive(Debug)]
pub enum ClientInitError {
    CertMissing(io::Error),
    CertInvalid(reqwest::Error),
    LeagueClientNotStarted(io::Error),
    LeagueClientInvalid(),
    LockfileAuthStringInvalid(io::Error),
    LockfileAuthHeaderInvalid(InvalidHeaderValue),
    ClientError(reqwest::Error),
}

impl From<CertificateError> for ClientInitError {
    fn from(cert_err: CertificateError) -> Self {
        match cert_err {
            CertificateError::Missing(err) => Self::CertMissing(err),
            CertificateError::Invalid(err) => Self::CertInvalid(err),
        }
    }
}

impl From<LockfileError> for ClientInitError {
    fn from(lf_error: LockfileError) -> Self {
        match lf_error {
            LockfileError::Missing(err) => Self::LeagueClientNotStarted(err),
            LockfileError::CantBeRead => Self::LeagueClientInvalid(),
        }
    }
}

impl From<io::Error> for ClientInitError {
    fn from(error: io::Error) -> Self {
        Self::LockfileAuthStringInvalid(error)
    }
}

impl From<InvalidHeaderValue> for ClientInitError {
    fn from(error: InvalidHeaderValue) -> Self {
        Self::LockfileAuthHeaderInvalid(error)
    }
}

impl From<reqwest::Error> for ClientInitError {
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(error)
    }
}

enum CertificateError {
    Missing(io::Error),
    Invalid(reqwest::Error),
}

impl From<io::Error> for CertificateError {
    fn from(error: io::Error) -> Self {
        CertificateError::Missing(error)
    }
}

impl From<reqwest::Error> for CertificateError {
    fn from(error: reqwest::Error) -> Self {
        CertificateError::Invalid(error)
    }
}

enum LockfileError {
    Missing(io::Error),
    CantBeRead,
}

impl From<io::Error> for LockfileError {
    fn from(error: io::Error) -> Self {
        LockfileError::Missing(error)
    }
}

#[derive(Debug)]
pub enum RequestError {
    ClientFailed(reqwest::Error),
    SummonerIdNeeded(),
    InvalidResponse(),
    ParsingFailed(json::Error),
}

impl From<reqwest::Error> for RequestError {
    fn from(error: reqwest::Error) -> Self {
        RequestError::ClientFailed(error)
    }
}

impl From<json::Error> for RequestError {
    fn from(error: json::Error) -> Self {
        RequestError::ParsingFailed(error)
    }
}
