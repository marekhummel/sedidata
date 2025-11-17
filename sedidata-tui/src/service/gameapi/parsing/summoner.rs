use json::JsonValue;

use crate::model::summoner::{RankedQueueStats, Summoner};

use super::ParsingError;

pub fn parse_summoner(json: &JsonValue) -> Result<Summoner, ParsingError> {
    if let JsonValue::Object(obj) = json {
        let summoner_id = obj["summonerId"]
            .as_u64()
            .ok_or(ParsingError::InvalidType("summonerId".into()))?;
        let puuid = obj["puuid"].as_str().ok_or(ParsingError::InvalidType("puuid".into()))?;
        let game_name = obj["gameName"]
            .as_str()
            .ok_or(ParsingError::InvalidType("gameName".into()))?;
        let tag_line = obj["tagLine"]
            .as_str()
            .ok_or(ParsingError::InvalidType("tagLine".into()))?;
        let level = obj["summonerLevel"]
            .as_u16()
            .ok_or(ParsingError::InvalidType("summonerLevel".into()))?;

        return Ok(Summoner {
            id: summoner_id.into(),
            puuid: puuid.to_string(),
            game_name: game_name.to_string(),
            tag_line: tag_line.to_string(),
            level,
        });
    }

    Err(ParsingError::InvalidType("root".into()))
}

pub fn parse_ranked_stats(json: &JsonValue) -> Result<Vec<RankedQueueStats>, ParsingError> {
    let mut stats = Vec::new();

    if let JsonValue::Object(obj) = json {
        if let JsonValue::Array(queues_array) = &obj["queues"] {
            for queue_json in queues_array {
                if let JsonValue::Object(queue) = queue_json {
                    let queue_type = queue["queueType"]
                        .as_str()
                        .ok_or(ParsingError::InvalidType("queueType".into()))?
                        .to_string();

                    // Skip non-SR queues
                    if queue_type.contains("TFT") {
                        continue;
                    }

                    let tier = queue["tier"]
                        .as_str()
                        .ok_or(ParsingError::InvalidType("tier".into()))?
                        .to_string();

                    let division = queue["division"]
                        .as_str()
                        .ok_or(ParsingError::InvalidType("division".into()))?
                        .to_string();

                    let league_points = queue["leaguePoints"]
                        .as_u32()
                        .ok_or(ParsingError::InvalidType("leaguePoints".into()))?;

                    let wins = queue["wins"].as_u32().ok_or(ParsingError::InvalidType("wins".into()))?;

                    let _losses = queue["losses"]
                        .as_u32()
                        .ok_or(ParsingError::InvalidType("losses".into()))?;

                    let _is_provisional = queue["isProvisional"]
                        .as_bool()
                        .ok_or(ParsingError::InvalidType("isProvisional".into()))?;

                    let highest_tier = queue["highestTier"].as_str().unwrap_or("").to_string();

                    let highest_division = queue["highestDivision"].as_str().unwrap_or("").to_string();

                    stats.push(RankedQueueStats {
                        queue_type,
                        tier,
                        division,
                        league_points,
                        wins,
                        _losses,
                        _is_provisional,
                        highest_tier,
                        highest_division,
                    });
                }
            }
        }
    }

    Ok(stats)
}
