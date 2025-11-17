use json::JsonValue;

use crate::model::{
    champselect::{ChampSelectPlayerInfo, ChampSelectSession},
    ids::ChampionId,
};

use super::ParsingError;

pub fn parse_champ_select(json: &JsonValue) -> Result<ChampSelectSession, ParsingError> {
    if let JsonValue::Object(obj) = json {
        // Queue
        let queue_id = obj["queueId"]
            .as_u16()
            .ok_or(ParsingError::InvalidType("queueId".into()))?;

        // Local Player Cell
        let local_player_cell_id = obj["localPlayerCellId"]
            .as_u8()
            .ok_or(ParsingError::InvalidType("localPlayerCellId".into()))?;

        // Benched (ARAM only)
        let bench_json = &obj["benchChampions"];
        let bench = parse_bench_champions(bench_json)?;

        // Teams
        let my_team = parse_team_players(&obj["myTeam"], true)?;
        let their_team = parse_team_players(&obj["theirTeam"], false)?;

        return Ok(ChampSelectSession {
            queue_id,
            local_player_cell: local_player_cell_id,
            benched_champs: bench,
            my_team,
            their_team,
        });
    }

    Err(ParsingError::InvalidType("root".into()))
}

fn parse_bench_champions(bench_json: &JsonValue) -> Result<Vec<ChampionId>, ParsingError> {
    let mut bench = Vec::new();

    if let JsonValue::Array(bench_array) = bench_json {
        for bench_champ_json in bench_array {
            if let JsonValue::Object(bench_champ) = bench_champ_json {
                let cid = bench_champ["championId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championId".into()))?;
                bench.push(cid.into())
            } else {
                return Err(ParsingError::InvalidType("benchChampions entry".into()));
            }
        }
        Ok(bench)
    } else {
        Err(ParsingError::InvalidType("benchChampions".into()))
    }
}

fn parse_team_players(team_json: &JsonValue, is_ally: bool) -> Result<Vec<ChampSelectPlayerInfo>, ParsingError> {
    let mut players = Vec::new();

    if let JsonValue::Array(team_array) = team_json {
        for player_json in team_array {
            if let JsonValue::Object(player) = player_json {
                // Check if player is hidden
                let name_visibility = player["nameVisibilityType"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("nameVisibilityType".into()))?;

                if name_visibility == "HIDDEN" {
                    // Skip hidden players entirely
                    continue;
                }

                let cell_id = player["cellId"]
                    .as_u8()
                    .ok_or(ParsingError::InvalidType("cellId".into()))?;

                let champ_id = player["championId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championId".into()))?;

                let position = player["assignedPosition"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("assignedPosition".into()))?
                    .to_string();

                let puuid = player["puuid"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("puuid".into()))?
                    .to_string();

                players.push(ChampSelectPlayerInfo {
                    cell_id,
                    position,
                    puuid,
                    is_ally,
                    selected_champion: champ_id.into(),
                });
            } else {
                return Err(ParsingError::InvalidType("team entry".into()));
            }
        }
        Ok(players)
    } else {
        Err(ParsingError::InvalidType("team".into()))
    }
}
