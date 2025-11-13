use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt,
    fs::{create_dir, File},
    io::{self, BufRead, Read, Write},
    path::Path,
    rc::Rc,
};

use base64::{engine::general_purpose, write::EncoderStringWriter};
use json::JsonValue;
use reqwest::{
    blocking::Client,
    header::{self, HeaderMap, HeaderValue, InvalidHeaderValue},
    Certificate,
};

use crate::model::summoner::Summoner;

#[derive(Debug)]
struct LockFileContent {
    base_url: String,
    username: String,
    password: String,
}

pub struct ApiClient {
    write_json: bool,
    load_local_json: bool,
    client: Client,
    cache: RefCell<HashMap<ClientRequestType, Rc<JsonValue>>>,
    base_url: String,
    summoner: Option<Summoner>,
}

impl ApiClient {
    pub fn new(read_json_files: bool, write_json: bool) -> Result<Self, ClientInitError> {
        let (client, base_url) = ApiClient::setup_client(read_json_files)?;
        let cache = RefCell::from(HashMap::new());
        Ok(Self {
            write_json,
            load_local_json: read_json_files,
            client,
            cache,
            base_url,
            summoner: None,
        })
    }

    fn setup_client(dummy: bool) -> Result<(Client, String), ClientInitError> {
        if dummy {
            let client = Client::builder().build()?;
            return Ok((client, String::new()));
        }

        // Read certificate
        let cert = ApiClient::read_certificate()?;

        // Read lockfile and create basic auth secret
        let lockfile = ApiClient::read_lockfile()?;
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

        Ok((client, lockfile.base_url))
    }

    fn read_certificate() -> Result<Certificate, CertificateError> {
        let mut buffer = Vec::new();
        let mut cert_file = File::open("config/riotgames.pem")?;
        cert_file.read_to_end(&mut buffer)?;
        let cert = reqwest::Certificate::from_pem(&buffer)?;
        Ok(cert)
    }

    fn read_lockfile() -> Result<LockFileContent, LockfileError> {
        // read config file
        let config_path_file = File::open("config/league_path.txt")?;
        let league_path = io::BufReader::new(config_path_file)
            .lines()
            .next()
            .ok_or(LockfileError::CantBeRead)??;

        // read lockfile
        let lol_path = Path::new(league_path.trim());
        let lol_lockfile = File::open(lol_path.join("lockfile"))?;
        let content = io::BufReader::new(lol_lockfile)
            .lines()
            .next()
            .ok_or(LockfileError::CantBeRead)??;

        // Grab content
        let info = content.split(':').collect::<Vec<_>>();
        Ok(LockFileContent {
            base_url: format!("{}://127.0.0.1:{}/", info[4], info[2]),
            username: "riot".to_string(),
            password: info[3].to_string(),
        })
    }

    pub fn request(&self, request_type: ClientRequestType, cache: bool) -> Result<Rc<JsonValue>, RequestError> {
        if self.load_local_json {
            let mut file = File::open(format!("data/{:?}.json", request_type))?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            let json = json::parse(buf.as_str()).unwrap();
            return Ok(Rc::new(json));
        }

        match self.cache.borrow_mut().entry(request_type) {
            Entry::Occupied(oe) => Ok(oe.get().clone()),
            Entry::Vacant(ve) => {
                // Get url
                let url = match request_type {
                    ClientRequestType::Summoner => {
                        format!("{}lol-summoner/v1/current-summoner", self.base_url)
                    }
                    ClientRequestType::Champions => match &self.summoner {
                        Some(s) => format!("{}lol-champions/v1/inventories/{}/champions", self.base_url, s.id),
                        None => return Err(RequestError::SummonerNeeded),
                    },
                    ClientRequestType::Masteries => match &self.summoner {
                        Some(_) => format!("{}lol-champion-mastery/v1/local-player/champion-mastery", self.base_url),
                        None => return Err(RequestError::SummonerNeeded),
                    },
                    ClientRequestType::Loot => {
                        format!("{}lol-loot/v1/player-loot", self.base_url)
                    }
                    ClientRequestType::ChampSelect => {
                        format!("{}lol-champ-select/v1/session", self.base_url)
                    }
                    ClientRequestType::Challenges => {
                        format!("{}lol-challenges/v1/challenges/local-player", self.base_url)
                    }
                };

                // Send request
                let response = self.client.get(url).send()?;
                if !response.status().is_success() {
                    return Err(RequestError::InvalidResponse(request_type, Box::new(response)));
                }

                // Return json
                let text = response.text()?;
                let json = json::parse(text.as_str())?;

                if self.write_json {
                    let _ = create_dir("data");
                    let mut file = File::create(format!("data/{:?}.json", request_type)).unwrap();
                    let _ = file.write_all(json.pretty(2).as_bytes());
                }

                let rc_json = Rc::new(json);
                if cache {
                    ve.insert(rc_json.clone());
                }
                Ok(rc_json)
            }
        }
    }

    pub fn set_summoner(&mut self, s: Summoner) {
        self.summoner = Some(s);
    }

    pub fn refresh(&mut self) -> Result<(), ClientInitError> {
        let (client, base_url) = ApiClient::setup_client(self.load_local_json)?;
        self.client = client;
        self.base_url = base_url;

        self.cache.borrow_mut().clear();
        self.summoner = None;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum ClientRequestType {
    Summoner,
    Champions,
    Masteries,
    Loot,
    ChampSelect,
    Challenges,
}

#[derive(Debug)]
pub enum ClientInitError {
    CertMissing(io::Error),
    CertInvalid(reqwest::Error),
    LeagueClientFailed(io::Error),
    LeagueClientInvalid(),
    LockfileAuthStringInvalid(io::Error),
    LockfileAuthHeaderInvalid(InvalidHeaderValue),
    ClientError(reqwest::Error),
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientInitError::CertMissing(err) => write!(f, "Certificate missing: {}", err),
            ClientInitError::CertInvalid(err) => write!(f, "Certificate invalid: {}", err),
            ClientInitError::LeagueClientFailed(err) => {
                write!(f, "League client failed, make sure it is running: {}", err)
            }
            ClientInitError::LeagueClientInvalid() => write!(f, "League client invalid lockfile."),
            ClientInitError::LockfileAuthStringInvalid(err) => write!(f, "Lockfile auth string invalid: {}", err),
            ClientInitError::LockfileAuthHeaderInvalid(err) => write!(f, "Lockfile auth header invalid: {}", err),
            ClientInitError::ClientError(err) => write!(f, "Client error: {}", err),
        }
    }
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
            LockfileError::Missing(err) => Self::LeagueClientFailed(err),
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
    SummonerNeeded,
    InvalidResponse(ClientRequestType, Box<reqwest::blocking::Response>),
    ParsingFailed(json::Error),
    LocalFileError(io::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::ClientFailed(err) => write!(f, "Client error: {}", err),
            RequestError::SummonerNeeded => write!(f, "Summoner information is needed for this request."),
            RequestError::InvalidResponse(req_type, response) => write!(
                f,
                "The server returned an invalid response for request {:?}: {:?}",
                req_type, response
            ),
            RequestError::ParsingFailed(err) => write!(f, "Parsing error: {}", err),
            RequestError::LocalFileError(err) => write!(f, "Local file error: {}", err),
        }
    }
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

impl From<io::Error> for RequestError {
    fn from(error: io::Error) -> Self {
        RequestError::LocalFileError(error)
    }
}
