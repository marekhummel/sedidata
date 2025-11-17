use json::JsonValue;

use crate::model::mastery::{Mastery, Milestone};

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
                let points_to_next_level = champ_obj["championPointsUntilNextLevel"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("championPointsUntilNextLevel".into()))?;

                let marks = champ_obj["tokensEarned"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("tokensEarned".into()))?;
                let required_marks = champ_obj["markRequiredForNextLevel"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("markRequiredForNextLevel".into()))?;

                let next_milestone = &champ_obj["nextSeasonMilestone"];
                let next_ms_reward = next_milestone["rewardMarks"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("nextSeasonMilestone/rewardMarks".into()))?;
                let next_ms_requirements = next_milestone["requireGradeCounts"]
                    .entries()
                    .map(|(grade, count)| {
                        let count_u16 = count
                            .as_u16()
                            .ok_or(ParsingError::InvalidType("requireGradeCounts/count".into()))?;
                        Ok((grade.to_string(), count_u16))
                    })
                    .collect::<Result<Vec<(String, u16)>, ParsingError>>()?;

                masteries.push(Mastery {
                    champ_id: champ_id.into(),
                    level,
                    points,
                    missing_points: points_to_next_level,
                    marks,
                    required_marks,
                    next_milestone: Milestone {
                        reward_marks: next_ms_reward,
                        require_grade_counts: next_ms_requirements,
                    },
                })
            } else {
                return Err(ParsingError::InvalidType("mastery entry".into()));
            }
        }

        return Ok(masteries);
    }

    Err(ParsingError::InvalidType("root".into()))
}
