use json::JsonValue;

use crate::model::{
    game::{PostGamePlayerInfo, PostGameSession, PostGameTeamInfo},
    summoner::SummonerName,
};

use super::ParsingError;

pub fn parse_post_game(json: &JsonValue) -> Result<PostGameSession, ParsingError> {
    if let JsonValue::Object(obj) = json {
        // Game ID
        let game_id = obj["gameId"]
            .as_u64()
            .ok_or(ParsingError::InvalidType("gameId".into()))?;

        // Teams
        let teams_json = &obj["teams"];
        let teams = parse_teams(teams_json)?;

        return Ok(PostGameSession { game_id, teams });
    }

    Err(ParsingError::InvalidType("root".into()))
}

fn parse_teams(teams_json: &JsonValue) -> Result<Vec<PostGameTeamInfo>, ParsingError> {
    let mut teams = Vec::new();

    if let JsonValue::Array(teams_array) = teams_json {
        for team_json in teams_array {
            if let JsonValue::Object(team) = team_json {
                let is_player_team = team["isPlayerTeam"]
                    .as_bool()
                    .ok_or(ParsingError::InvalidType("isPlayerTeam".into()))?;

                let is_winning_team = team["isWinningTeam"]
                    .as_bool()
                    .ok_or(ParsingError::InvalidType("isWinningTeam".into()))?;

                let players_json = &team["players"];
                let players = parse_players(players_json)?;

                teams.push(PostGameTeamInfo {
                    is_player_team,
                    _is_winning_team: is_winning_team,
                    players,
                });
            } else {
                return Err(ParsingError::InvalidType("teams entry".into()));
            }
        }
        Ok(teams)
    } else {
        Err(ParsingError::InvalidType("teams".into()))
    }
}

fn parse_players(players_json: &JsonValue) -> Result<Vec<PostGamePlayerInfo>, ParsingError> {
    let mut players = Vec::new();

    if let JsonValue::Array(players_array) = players_json {
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

                // Use detectedTeamPosition, fallback to selectedPosition
                let position = player["detectedTeamPosition"]
                    .as_str()
                    .or_else(|| player["selectedPosition"].as_str())
                    .ok_or(ParsingError::InvalidType(
                        "detectedTeamPosition/selectedPosition".into(),
                    ))?
                    .to_string();

                let champion_name = player["championName"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("championName".into()))?
                    .to_string();

                let team_id = player["teamId"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("teamId".into()))?;

                let is_bot = player["botPlayer"]
                    .as_bool()
                    .ok_or(ParsingError::InvalidType("botPlayer".into()))?;

                players.push(PostGamePlayerInfo {
                    name: SummonerName { game_name, tag_line },
                    position,
                    champion_name,
                    team_id,
                    _is_bot: is_bot,
                });
            } else {
                return Err(ParsingError::InvalidType("players entry".into()));
            }
        }
        Ok(players)
    } else {
        Err(ParsingError::InvalidType("players".into()))
    }
}
