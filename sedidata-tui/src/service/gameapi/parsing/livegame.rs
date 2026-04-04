use json::JsonValue;

use crate::model::{
    game::{LiveGamePlayerInfo, LiveGameSession},
    summoner::SummonerName,
};

use super::ParsingError;

pub fn parse_live_game(json: &JsonValue) -> Result<LiveGameSession, ParsingError> {
    if let JsonValue::Object(obj) = json {
        return Ok(LiveGameSession {
            players: parse_live_game_players(&obj["allPlayers"])?,
            game_mode: parse_live_game_gametype(&obj["gameData"])?,
        });
    }

    Err(ParsingError::InvalidType("root".into()))
}

fn parse_live_game_players(players_json: &JsonValue) -> Result<Vec<LiveGamePlayerInfo>, ParsingError> {
    if let JsonValue::Array(players_array) = players_json {
        let mut players = Vec::new();

        for player_json in players_array {
            if let JsonValue::Object(player) = player_json {
                let riot_id = player["riotId"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("riotIdGameName".into()))?;

                let summoner_name = match riot_id {
                    "#" => None,
                    _ => {
                        let game_name = player["riotIdGameName"]
                            .as_str()
                            .ok_or(ParsingError::InvalidType("riotIdGameName".into()))?
                            .to_string();

                        let tag_line = player["riotIdTagLine"]
                            .as_str()
                            .ok_or(ParsingError::InvalidType("riotIdTagLine".into()))?
                            .to_string();
                        Some(SummonerName { game_name, tag_line })
                    }
                };

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
                    name: summoner_name,
                    position,
                    champion_name,
                    team,
                    _is_bot: is_bot,
                });
            } else {
                return Err(ParsingError::InvalidType("player entry".into()));
            }
        }

        Ok(players)
    } else {
        Err(ParsingError::InvalidType("root (expected player array)".into()))
    }
}

fn parse_live_game_gametype(json: &JsonValue) -> Result<Option<String>, ParsingError> {
    if let JsonValue::Object(obj) = json {
        return Ok(Some(
            obj["gameMode"]
                .as_str()
                .ok_or(ParsingError::InvalidType("gameMode".into()))?
                .to_string(),
        ));
    }

    Err(ParsingError::InvalidType("gameMode".into()))
}
