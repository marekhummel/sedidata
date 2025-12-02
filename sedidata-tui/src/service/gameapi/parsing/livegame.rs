use json::JsonValue;

use crate::model::game::{LiveGamePlayerInfo, LiveGameSession};

use super::ParsingError;

pub fn parse_live_game(json: &JsonValue) -> Result<LiveGameSession, ParsingError> {
    if let JsonValue::Array(players_array) = json {
        let mut players = Vec::new();

        for player_json in players_array {
            if let JsonValue::Object(player) = player_json {
                let game_name = player["riotIdGameName"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("riotIdGameName".into()))?
                    .to_string();

                let tag_line = player["riotIdTagLine"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("riotIdTagLine".into()))?
                    .to_string();

                let position = player["position"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("position".into()))?
                    .to_string();

                let champion_name = player["championName"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("championName".into()))?
                    .to_string();

                let team = player["team"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("team".into()))?
                    .to_string();

                let is_bot = player["isBot"]
                    .as_bool()
                    .ok_or(ParsingError::InvalidType("isBot".into()))?;

                players.push(LiveGamePlayerInfo {
                    game_name,
                    tag_line,
                    position,
                    champion_name,
                    team,
                    _is_bot: is_bot,
                });
            } else {
                return Err(ParsingError::InvalidType("player entry".into()));
            }
        }

        Ok(LiveGameSession { players })
    } else {
        Err(ParsingError::InvalidType("root (expected array)".into()))
    }
}
