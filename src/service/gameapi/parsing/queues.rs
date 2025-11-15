use json::JsonValue;

use crate::model::champselect::QueueInfo;

use super::ParsingError;

pub fn parse_queues(json: &JsonValue) -> Result<Vec<QueueInfo>, ParsingError> {
    let mut queues: Vec<QueueInfo> = Vec::new();

    for queue in json.members() {
        if let JsonValue::Object(queue_obj) = &queue {
            // Challenge
            let queue_id = queue_obj["id"].as_u16().ok_or(ParsingError::InvalidType("id".into()))?;

            let _category = queue_obj["category"]
                .as_str()
                .ok_or(ParsingError::InvalidType("category".into()))?
                .to_string();
            let _description = queue_obj["description"]
                .as_str()
                .ok_or(ParsingError::InvalidType("description".into()))?
                .to_string();
            let gamemode = queue_obj["gameMode"]
                .as_str()
                .ok_or(ParsingError::InvalidType("gameMode".into()))?
                .to_string();

            let _type_descriptor = queue_obj["type"]
                .as_str()
                .ok_or(ParsingError::InvalidType("type".into()))?
                .to_string();

            queues.push(QueueInfo {
                queue_id,
                _category,
                _description,
                gamemode,
                _type_descriptor,
            });
        } else {
            return Err(ParsingError::InvalidType("children".into()));
        }
    }

    Ok(queues)
}
