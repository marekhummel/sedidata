use json::JsonValue;

use crate::model::summoner::Summoner;

use super::ParsingError;

pub fn parse_summoner(json: &JsonValue) -> Result<Summoner, ParsingError> {
    if let JsonValue::Object(obj) = json {
        let summoner_id = obj["summonerId"]
            .as_u64()
            .ok_or(ParsingError::InvalidType("summonerId".into()))?;
        let display_name = obj["displayName"]
            .as_str()
            .ok_or(ParsingError::InvalidType("displayName".into()))?;
        let internal_name = obj["internalName"]
            .as_str()
            .ok_or(ParsingError::InvalidType("internalName".into()))?;
        let level = obj["summonerLevel"]
            .as_u16()
            .ok_or(ParsingError::InvalidType("summonerLevel".into()))?;

        return Ok(Summoner {
            id: summoner_id.into(),
            display_name: display_name.to_string(),
            internal_name: internal_name.to_string(),
            level,
        });
    }

    Err(ParsingError::InvalidType("root".into()))
}
