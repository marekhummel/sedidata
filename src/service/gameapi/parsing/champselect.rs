use std::collections::HashMap;

use json::JsonValue;

use crate::model::{
    champselect::{ChampSelectInfo, SelectableChamp},
    ids::ChampionId,
};

use super::ParsingError;

pub fn parse_champselect_info(json: &JsonValue) -> Result<ChampSelectInfo, ParsingError> {
    if let JsonValue::Object(obj) = json {
        // Benched
        let bench_json = &obj["benchChampions"];
        let bench = parse_bench_champions(bench_json)?;

        // Picked
        let mut my_team = parse_my_team(&obj["myTeam"])?;
        let mut trades = parse_trades(&obj["trades"])?;

        let local_player_cell_id = obj["localPlayerCellId"]
            .as_u8()
            .ok_or(ParsingError::InvalidType("localPlayerCellId".into()))?;
        let current_champ_id = my_team
            .remove(&local_player_cell_id)
            .ok_or(ParsingError::InvalidType("localPlayerCellId".into()))?;

        let team_champs = my_team
            .into_iter()
            .map(|(cell, champ)| SelectableChamp {
                id: champ,
                selectable: trades.remove(&cell).unwrap_or(false),
            })
            .collect();

        return Ok(ChampSelectInfo {
            current_champ_id,
            team_champs,
            benched_champs: bench,
        });
    }

    Err(ParsingError::InvalidType("root".into()))
}

fn parse_bench_champions(bench_json: &JsonValue) -> Result<Vec<SelectableChamp>, ParsingError> {
    let mut bench = Vec::new();

    if let JsonValue::Array(bench_array) = bench_json {
        for bench_champ_json in bench_array {
            if let JsonValue::Object(bench_champ) = bench_champ_json {
                let cid = bench_champ["championId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championId".into()))?;

                let champ = SelectableChamp {
                    id: cid.into(),
                    selectable: true,
                };
                bench.push(champ)
            } else {
                return Err(ParsingError::InvalidType("benchChampions entry".into()));
            }
        }
        Ok(bench)
    } else {
        Err(ParsingError::InvalidType("benchChampions".into()))
    }
}

fn parse_my_team(my_team_json: &JsonValue) -> Result<HashMap<u8, ChampionId>, ParsingError> {
    let mut team = HashMap::new();

    if let JsonValue::Array(team_array) = my_team_json {
        for team_member_json in team_array {
            if let JsonValue::Object(team_member) = team_member_json {
                let cell_id = team_member["cellId"]
                    .as_u8()
                    .ok_or(ParsingError::InvalidType("cellId".into()))?;
                let champ_id = team_member["championId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championId".into()))?;

                team.insert(cell_id, champ_id.into());
            } else {
                return Err(ParsingError::InvalidType("myTeam entry".into()));
            }
        }
        Ok(team)
    } else {
        Err(ParsingError::InvalidType("myTeam".into()))
    }
}

fn parse_trades(trades_json: &JsonValue) -> Result<HashMap<u8, bool>, ParsingError> {
    let mut team = HashMap::new();

    if let JsonValue::Array(trades_array) = trades_json {
        for trade_json in trades_array {
            if let JsonValue::Object(trade) = trade_json {
                let cell_id = trade["cellId"]
                    .as_u8()
                    .ok_or(ParsingError::InvalidType("cellId".into()))?;
                let state = trade["state"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("state".into()))?;

                team.insert(cell_id, state == "AVAILABLE");
            } else {
                return Err(ParsingError::InvalidType("trades entry".into()));
            }
        }
        Ok(team)
    } else {
        Err(ParsingError::InvalidType("trades".into()))
    }
}
