use std::collections::HashMap;

use json::JsonValue;

use crate::model::challenge::{Challenge, ChallengeCategory, Threshold};

use super::ParsingError;

pub fn parse_challenges(json: &JsonValue) -> Result<Vec<ChallengeCategory>, ParsingError> {
    let mut categories: HashMap<i32, ChallengeCategory> = HashMap::new();
    let mut challenges: Vec<Challenge> = Vec::new();

    for (_, entry) in json.entries() {
        if let JsonValue::Object(challenge_obj) = &entry {
            let challenge_id = challenge_obj["id"]
                .as_i32()
                .ok_or(ParsingError::InvalidType("id".into()))?;

            let name = challenge_obj["name"]
                .as_str()
                .ok_or(ParsingError::InvalidType("name".into()))?
                .to_string();

            if let JsonValue::Array(children_array) = &challenge_obj["childrenIds"] {
                // let children_ids: Vec<_> = children_array.iter().filter_map(|child| child.as_i16()).collect();

                if !children_array.is_empty() {
                    // Category
                    categories.insert(
                        challenge_id,
                        ChallengeCategory {
                            id: challenge_id,
                            name,
                            children: Vec::new(),
                        },
                    );
                } else {
                    // Challenge
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

                    const LEVELS: [&str; 9] = [
                        "IRON",
                        "BRONZE",
                        "SILVER",
                        "GOLD",
                        "PLATINUM",
                        "DIAMOND",
                        "MASTER",
                        "GRANDMASTER",
                        "CHALLENGER",
                    ];

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
                        current_level,
                        next_level,
                        thresholds,
                        parent_id,
                    });
                }
            } else {
                return Err(ParsingError::InvalidType("children".into()));
            }
        } else {
            return Err(ParsingError::InvalidType("challenge entry".into()));
        }
    }

    for challenge in challenges {
        if let Some(category) = categories.get_mut(&(challenge.parent_id)) {
            category.children.push(challenge);
        } else {
            if challenge.parent_id == -1 {
                categories.insert(
                    -1,
                    ChallengeCategory {
                        id: -1,
                        name: "MISC".into(),
                        children: Vec::from([challenge]),
                    },
                );
                continue;
            }

            return Err(ParsingError::InvalidType(format!(
                "invalid parent id {}",
                challenge.parent_id
            )));
        }
    }

    Ok(categories.into_values().collect())
}
