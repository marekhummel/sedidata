use chrono::{TimeZone, Utc};
use json::{object::Object, JsonValue};

use crate::model::games::{Game, QueueType, Statistics};

use super::ParsingError;

pub fn parse_game_stats(json: &JsonValue) -> Result<Vec<Game>, ParsingError> {
    if let JsonValue::Array(array) = json {
        let mut games = Vec::new();

        for game_entry in array {
            if let JsonValue::Object(game_obj) = &game_entry {
                let game_opt = parse_game_obj(game_obj)?;
                if let Some(game) = game_opt {
                    games.push(game)
                }
            } else {
                return Err(ParsingError::InvalidType("game entry".into()));
            }
        }

        return Ok(games);
    }

    Err(ParsingError::InvalidType("root".into()))
}

fn parse_game_obj(obj: &Object) -> Result<Option<Game>, ParsingError> {
    let champ_id = obj["championId"]
        .as_i32()
        .ok_or(ParsingError::InvalidType("championId".into()))?;
    let queue = obj["queueType"]
        .as_str()
        .ok_or(ParsingError::InvalidType("queueType".into()))?;
    let season = obj["season"]
        .as_u8()
        .ok_or(ParsingError::InvalidType("season".into()))?;
    let timestamp = obj["timestamp"]
        .as_i64()
        .ok_or(ParsingError::InvalidType("timestamp".into()))?;

    let queue_type = match queue {
        "blind5" => QueueType::Blind,
        "draft5" => QueueType::Draft,
        "rank5flex" => QueueType::RankedFlex,
        "rank5solo" => QueueType::RankedSolo,
        "rank3flex" => return Ok(None),
        _ => return Err(ParsingError::InvalidType(format!("queueType '{}'", queue))),
    };

    let stats_json = &obj["stats"]["CareerStats.js"];
    match &stats_json {
        JsonValue::Object(stats_obj) => {
            let stats = parse_game_stats_obj(stats_obj)?;
            Ok(Some(Game {
                champ_id: champ_id.into(),
                queue: queue_type,
                season,
                timestamp: Utc.timestamp_millis_opt(timestamp).unwrap(),
                stats,
            }))
        }
        _ => Err(ParsingError::InvalidType("stats/CareerStats.js".into())),
    }
}

fn parse_game_stats_obj(obj: &Object) -> Result<Statistics, ParsingError> {
    let kills = obj["kills"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("kills".into()))?;
    let deaths = obj["deaths"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("deaths".into()))?;
    let assists = obj["assists"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("assists".into()))?;
    let doubles = obj["doubleKills"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("doubleKills".into()))?;
    let triples = obj["tripleKills"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("tripleKills".into()))?;
    let quadras = obj["quadraKills"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("quadraKills".into()))?;
    let pentas = obj["pentaKills"]
        .as_f32()
        .ok_or(ParsingError::InvalidType("pentaKills".into()))?;

    Ok(Statistics {
        kills: kills as u16,
        deaths: deaths as u16,
        assists: assists as u16,
        doubles: doubles as u16,
        triples: triples as u16,
        quadras: quadras as u16,
        pentas: pentas as u16,
    })
}
