use std::{fmt, fs::File, io::Read};

use json::JsonValue;
use reqwest::blocking::Client;

pub struct LiveGameClient {
    write_json: bool,
    load_local_json: bool,
    client: Client,
    base_url: String,
}

impl LiveGameClient {
    pub fn new(read_json_files: bool, write_json: bool) -> Self {
        let client = Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let base_url = "https://127.0.0.1:2999".to_string();

        Self {
            write_json,
            load_local_json: read_json_files,
            client,
            base_url,
        }
    }

    pub fn request(&self) -> Result<JsonValue, LiveGameRequestError> {
        if self.load_local_json {
            let mut file = File::open("data/Playerlist.json")?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            let json = json::parse(buf.as_str())?;
            return Ok(json);
        }

        let url = format!("{}/liveclientdata/playerlist", self.base_url);
        let response = self.client.get(url).send()?;

        if !response.status().is_success() {
            return Err(LiveGameRequestError::InvalidResponse(response.status()));
        }

        let text = response.text()?;
        let json = json::parse(text.as_str())?;

        if self.write_json {
            if let Err(e) = std::fs::create_dir("data") {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(LiveGameRequestError::LocalFileError(e));
                }
            }
            let mut file = File::create("data/Playerlist.json")?;
            use std::io::Write;
            file.write_all(json.pretty(2).as_bytes())?;
        }

        Ok(json)
    }
}

#[derive(Debug)]
pub enum LiveGameRequestError {
    ClientFailed(reqwest::Error),
    InvalidResponse(reqwest::StatusCode),
    ParsingFailed(json::Error),
    LocalFileError(std::io::Error),
}

impl fmt::Display for LiveGameRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LiveGameRequestError::ClientFailed(err) => write!(f, "Client error: {}", err),
            LiveGameRequestError::InvalidResponse(status) => write!(f, "Invalid response: {}", status),
            LiveGameRequestError::ParsingFailed(err) => write!(f, "Parsing error: {}", err),
            LiveGameRequestError::LocalFileError(err) => write!(f, "Local file error: {}", err),
        }
    }
}

impl From<reqwest::Error> for LiveGameRequestError {
    fn from(error: reqwest::Error) -> Self {
        LiveGameRequestError::ClientFailed(error)
    }
}

impl From<json::Error> for LiveGameRequestError {
    fn from(error: json::Error) -> Self {
        LiveGameRequestError::ParsingFailed(error)
    }
}

impl From<std::io::Error> for LiveGameRequestError {
    fn from(error: std::io::Error) -> Self {
        LiveGameRequestError::LocalFileError(error)
    }
}
