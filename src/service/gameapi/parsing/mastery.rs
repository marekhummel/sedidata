use json::JsonValue;

use crate::model::mastery::Mastery;

use super::ParsingError;

pub fn parse_masteries(json: &JsonValue) -> Result<Vec<Mastery>, ParsingError> {
    if let JsonValue::Array(array) = json {
        let mut masteries = Vec::new();

        for champ_entry in array {
            if let JsonValue::Object(champ_obj) = &champ_entry {
                let champ_id = champ_obj["championId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championId".into()))?;
                let level = champ_obj["championLevel"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("championLevel".into()))?;
                let points = champ_obj["championPoints"]
                    .as_u32()
                    .ok_or(ParsingError::InvalidType("championPoints".into()))?;
                let tokens = champ_obj["tokensEarned"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("tokensEarned".into()))?;
                let points_to_next_level = champ_obj["championPointsUntilNextLevel"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championPointsUntilNextLevel".into()))?;

                masteries.push(Mastery {
                    champ_id: champ_id.into(),
                    level,
                    points,
                    tokens: if level == 5 || level == 6 { Some(tokens) } else { None },
                    points_to_next_level,
                    // chest_granted: false,
                })
            } else {
                return Err(ParsingError::InvalidType("mastery entry".into()));
            }
        }

        return Ok(masteries);
    }

    Err(ParsingError::InvalidType("root".into()))
}
