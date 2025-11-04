use json::{object::Object, JsonValue};

use crate::model::{
    champion::{AllChampionInfo, Champion, Chroma, Skin},
    ids::SkinId,
};

use super::ParsingError;

pub fn parse_champions(json: &JsonValue) -> Result<AllChampionInfo, ParsingError> {
    if let JsonValue::Array(array) = json {
        let mut champions = Vec::new();
        let mut skins = Vec::new();
        let mut chromas = Vec::new();

        for champ_entry in array {
            if let JsonValue::Object(champ_obj) = &champ_entry {
                let champ = parse_champ_obj(champ_obj)?;
                if champ.id == String::from("-1").into() || champ.name.contains("Doom Bot") {
                    continue;
                }

                champions.push(champ);
                let champ_skins = &champ_obj["skins"];
                if let JsonValue::Array(skin_array) = champ_skins {
                    for skin_entry in skin_array {
                        if let JsonValue::Object(skin_obj) = &skin_entry {
                            let skin = parse_skin_obj(skin_obj)?;
                            let skin_id = skin.id.clone();
                            skins.push(skin);

                            let skin_chromas = &skin_obj["chromas"];
                            if let JsonValue::Array(chroma_array) = skin_chromas {
                                for chroma_entry in chroma_array {
                                    if let JsonValue::Object(chroma_obj) = &chroma_entry {
                                        let chroma = parse_chroma_obj(chroma_obj, skin_id.clone())?;
                                        chromas.push(chroma);
                                    } else {
                                        return Err(ParsingError::InvalidType("chroma entry".into()));
                                    }
                                }
                            } else {
                                return Err(ParsingError::InvalidType("chromas".into()));
                            }
                        } else {
                            return Err(ParsingError::InvalidType("skin entry".into()));
                        }
                    }
                } else {
                    return Err(ParsingError::InvalidType("skins".into()));
                }
            } else {
                return Err(ParsingError::InvalidType("champ entry".into()));
            }
        }

        return Ok(AllChampionInfo {
            champions,
            skins,
            chromas,
        });
    }

    Err(ParsingError::InvalidType("root".into()))
}

fn parse_champ_obj(obj: &Object) -> Result<Champion, ParsingError> {
    let champ_id = obj["id"].as_i32().ok_or(ParsingError::InvalidType("id".into()))?;
    let name = obj["name"].as_str().ok_or(ParsingError::InvalidType("name".into()))?;
    let owned = obj["ownership"]["owned"]
        .as_bool()
        .ok_or(ParsingError::InvalidType("ownership/owned".into()))?;

    Ok(Champion {
        id: champ_id.into(),
        name: name.to_string(),
        owned,
    })
}

fn parse_skin_obj(obj: &Object) -> Result<Skin, ParsingError> {
    let skin_id = obj["id"].as_i32().ok_or(ParsingError::InvalidType("id".into()))?;
    let champion_id = obj["championId"]
        .as_i32()
        .ok_or(ParsingError::InvalidType("championId".into()))?;
    let name = obj["name"].as_str().ok_or(ParsingError::InvalidType("name".into()))?;
    let is_base = obj["isBase"]
        .as_bool()
        .ok_or(ParsingError::InvalidType("isBase".into()))?;
    let owned = obj["ownership"]["owned"]
        .as_bool()
        .ok_or(ParsingError::InvalidType("ownership/owned".into()))?;

    Ok(Skin {
        id: skin_id.into(),
        champ_id: champion_id.into(),
        name: name.to_string(),
        is_base,
        owned,
    })
}

fn parse_chroma_obj(obj: &Object, skin_id: SkinId) -> Result<Chroma, ParsingError> {
    let chroma_id = obj["id"].as_i32().ok_or(ParsingError::InvalidType("id".into()))?;
    let owned = obj["ownership"]["owned"]
        .as_bool()
        .ok_or(ParsingError::InvalidType("ownership/owned".into()))?;

    Ok(Chroma {
        id: chroma_id.into(),
        skin_id,
        owned,
    })
}
