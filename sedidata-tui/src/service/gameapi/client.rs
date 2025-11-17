use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    env, fmt,
    fs::{create_dir, File},
    io::{self, BufRead, Read, Write},
    path::{Path, PathBuf},
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
        let league_install_path = ApiClient::get_or_prompt_league_path()?;
        let (client, base_url) = ApiClient::setup_client(&league_install_path, read_json_files)?;
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

    fn get_app_data_dir() -> Result<PathBuf, ClientInitError> {
        let local_app_data = env::var("LOCALAPPDATA").map_err(|_| ClientInitError::LocalAppDataNotFound)?;

        let mut app_dir = PathBuf::from(local_app_data);
        app_dir.push("sedidata");

        // Create directory if it doesn't exist
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir).map_err(ClientInitError::AppDataDirCreationFailed)?;
        }

        Ok(app_dir)
    }

    fn get_or_prompt_league_path() -> Result<String, ClientInitError> {
        let app_dir = ApiClient::get_app_data_dir()?;
        let path_file = app_dir.join("league_path.txt");

        // Try to read existing path
        if path_file.exists() {
            if let Ok(mut file) = File::open(&path_file) {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    let path = content.trim().to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
            }
        }

        // Prompt user for path
        println!("\n=== League of Legends Installation Path Setup ===");
        println!("Please enter the path to your League of Legends installation directory.");
        println!("(Press Enter to use default: C:\\Program Files\\Riot Games\\League of Legends\\)");
        print!("\nPath: ");
        io::Write::flush(&mut io::stdout()).unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(ClientInitError::UserInputFailed)?;

        let league_path = input.trim();
        let league_path = if league_path.is_empty() {
            r"C:\Program Files\Riot Games\League of Legends\"
        } else {
            league_path
        };

        // Verify the path exists
        let path = Path::new(league_path);
        if !path.exists() {
            return Err(ClientInitError::LeaguePathInvalid(format!(
                "Path does not exist: {}",
                league_path
            )));
        }

        // Save the path
        let mut file = File::create(&path_file).map_err(ClientInitError::PathFileSaveFailed)?;
        file.write_all(league_path.as_bytes())
            .map_err(ClientInitError::PathFileSaveFailed)?;

        println!("Path saved successfully!\n");
        Ok(league_path.to_string())
    }

    fn setup_client(league_install_path: &str, dummy: bool) -> Result<(Client, String), ClientInitError> {
        if dummy {
            let client = Client::builder().build()?;
            return Ok((client, String::new()));
        }

        // Read certificate
        let cert = ApiClient::read_certificate()?;

        // Read lockfile and create basic auth secret
        let lockfile = ApiClient::read_lockfile(league_install_path)?;
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
        // Get LocalAppData path
        let local_app_data = env::var("LOCALAPPDATA").map_err(|_| CertificateError::LocalAppDataNotFound)?;

        let mut cert_path = PathBuf::from(local_app_data);
        cert_path.push("sedidata");

        if !cert_path.exists() {
            std::fs::create_dir_all(&cert_path)?;
        }
        cert_path.push("riotgames.pem");

        // Download if missing
        if !cert_path.exists() {
            const CERT_URL: &str = "https://static.developer.riotgames.com/docs/lol/riotgames.pem";
            let client = Client::builder().build()?;
            let response = client.get(CERT_URL).send()?;

            if !response.status().is_success() {
                return Err(CertificateError::DownloadFailed(format!(
                    "Failed to download certificate: HTTP {}",
                    response.status()
                )));
            }

            let cert_data = response.bytes()?;
            let mut file = File::create(&cert_path)?;
            file.write_all(&cert_data)?;
        }

        // Load
        let mut buffer = Vec::new();
        let mut cert_file = File::open(&cert_path)?;
        cert_file.read_to_end(&mut buffer)?;
        let cert = reqwest::Certificate::from_pem(&buffer)?;
        Ok(cert)
    }

    fn read_lockfile(league_install_path: &str) -> Result<LockFileContent, LockfileError> {
        // read lockfile
        let lol_path = Path::new(league_install_path.trim());
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
            let mut file = File::open(format!("data/{}.json", request_type))?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            let json = json::parse(buf.as_str()).unwrap();
            return Ok(Rc::new(json));
        }

        match self.cache.borrow_mut().entry(request_type.clone()) {
            Entry::Occupied(oe) => Ok(oe.get().clone()),
            Entry::Vacant(ve) => {
                // Get url
                let url = match &request_type {
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
                    ClientRequestType::QueueTypes => {
                        format!("{}lol-game-queues/v1/queues", self.base_url)
                    }
                    ClientRequestType::OtherSummoner(puuid) => {
                        format!("{}lol-summoner/v2/summoners/puuid/{}", self.base_url, puuid)
                    }
                    ClientRequestType::RankedStats(puuid) => {
                        format!("{}lol-ranked/v1/ranked-stats/{}", self.base_url, puuid)
                    }
                };

                // Send request
                let response = self.client.get(url).send()?;
                if !response.status().is_success() {
                    return Err(RequestError::InvalidResponse(request_type.clone(), Box::new(response)));
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
        let league_install_path = ApiClient::get_or_prompt_league_path()?;
        let (client, base_url) = ApiClient::setup_client(&league_install_path, self.load_local_json)?;
        self.client = client;
        self.base_url = base_url;

        self.cache.borrow_mut().clear();
        self.summoner = None;
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ClientRequestType {
    Summoner,
    Champions,
    Masteries,
    Loot,
    ChampSelect,
    Challenges,
    QueueTypes,
    OtherSummoner(String), // PUUID parameter
    RankedStats(String),   // PUUID parameter
}

impl fmt::Display for ClientRequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientRequestType::Summoner => write!(f, "Summoner"),
            ClientRequestType::Champions => write!(f, "Champions"),
            ClientRequestType::Masteries => write!(f, "Masteries"),
            ClientRequestType::Loot => write!(f, "Loot"),
            ClientRequestType::ChampSelect => write!(f, "ChampSelect"),
            ClientRequestType::Challenges => write!(f, "Challenges"),
            ClientRequestType::QueueTypes => write!(f, "QueueTypes"),
            ClientRequestType::OtherSummoner(puuid) => write!(f, "OtherSummoner_{}", puuid),
            ClientRequestType::RankedStats(puuid) => write!(f, "RankedStats_{}", puuid),
        }
    }
}

#[derive(Debug)]
pub enum ClientInitError {
    CertMissing(io::Error),
    CertInvalid(reqwest::Error),
    CertDownloadFailed(String),
    LocalAppDataNotFound,
    AppDataDirCreationFailed(io::Error),
    LeaguePathInvalid(String),
    UserInputFailed(io::Error),
    PathFileSaveFailed(io::Error),
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
            ClientInitError::CertDownloadFailed(msg) => write!(f, "Certificate download failed: {}", msg),
            ClientInitError::LocalAppDataNotFound => write!(f, "Could not find LocalAppData directory"),
            ClientInitError::AppDataDirCreationFailed(err) => write!(f, "Failed to create app data directory: {}", err),
            ClientInitError::LeaguePathInvalid(msg) => write!(f, "Invalid League path: {}", msg),
            ClientInitError::UserInputFailed(err) => write!(f, "Failed to read user input: {}", err),
            ClientInitError::PathFileSaveFailed(err) => write!(f, "Failed to save path file: {}", err),
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
            CertificateError::DownloadFailed(msg) => Self::CertDownloadFailed(msg),
            CertificateError::LocalAppDataNotFound => Self::LocalAppDataNotFound,
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
    DownloadFailed(String),
    LocalAppDataNotFound,
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
