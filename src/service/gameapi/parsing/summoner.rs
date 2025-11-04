use json::JsonValue;

use crate::model::summoner::Summoner;

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
