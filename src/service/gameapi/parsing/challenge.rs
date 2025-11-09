use itertools::Itertools;
use json::JsonValue;

use crate::model::challenge::{Challenge, Threshold, LEVELS};

use super::ParsingError;

pub fn parse_challenges(json: &JsonValue) -> Result<Vec<Challenge>, ParsingError> {
    let mut challenges: Vec<Challenge> = Vec::new();

    for (_, entry) in json.entries() {
        if let JsonValue::Object(challenge_obj) = &entry {
            if let JsonValue::Array(children_array) = &challenge_obj["childrenIds"] {
                // Challenge
                let challenge_id = challenge_obj["id"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("id".into()))?;

                let name = challenge_obj["name"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("name".into()))?
                    .to_string();
                let description = challenge_obj["description"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("description".into()))?
                    .replace("<em>", "")
                    .replace("</em>", "")
                    .to_string();

                let children_ids: Vec<_> = children_array.iter().filter_map(|child| child.as_i32()).collect();
                let is_capstone = challenge_obj["isCapstone"]
                    .as_bool()
                    .ok_or(ParsingError::InvalidType("isCapstone".into()))?;
                let category = challenge_obj["category"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("category".into()))?
                    .to_string();
                let gamemodes = challenge_obj["gameModes"]
                    .members()
                    .map(|gm| gm.to_string())
                    .unique()
                    .collect::<Vec<_>>();

                let parent_id = challenge_obj["parentId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType(format!("parentId of {}", challenge_id)))?;
                let current_level = challenge_obj["currentLevel"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("currentLevel".into()))?
                    .to_string();
                let next_level = challenge_obj["nextLevel"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("nextLevel".into()))?
                    .to_string();

                let current_value = challenge_obj["currentValue"]
                    .as_f32()
                    .ok_or(ParsingError::InvalidType("currentValue".into()))?;
                let threshold_value = challenge_obj["nextThreshold"]
                    .as_f32()
                    .ok_or(ParsingError::InvalidType("nextThreshold".into()))?;

                let threshold_obj = &challenge_obj["thresholds"];
                let mut thresholds = Vec::new();
                for level in LEVELS {
                    if !threshold_obj.has_key(level) {
                        continue;
                    }
                    if let JsonValue::Array(rewards) = &threshold_obj[level]["rewards"] {
                        for reward in rewards {
                            if reward["category"].as_str().unwrap() != "CHALLENGE_POINTS" {
                                continue;
                            }

                            thresholds.push(Threshold {
                                level: level.to_string(),
                                value: reward["quantity"]
                                    .as_u16()
                                    .ok_or(ParsingError::InvalidType("threshold value".into()))?,
                            });
                        }
                    }
                }

                challenges.push(Challenge {
                    id: challenge_id,
                    name,
                    description,
                    current_level,
                    next_level,
                    current_value,
                    threshold_value,
                    thresholds,
                    gamemodes,
                    _parent_id: parent_id,
                    _children: children_ids,
                    is_capstone,
                    category,
                });
            }
        } else {
            return Err(ParsingError::InvalidType("children".into()));
        }
    }

    Ok(challenges)
}
