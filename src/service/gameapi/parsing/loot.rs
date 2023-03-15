use json::JsonValue;

use crate::model::loot::{
    ChampionShard, Credits, JsonLootItem, LootItems, MasteryToken, SkinShard,
};

use super::ParsingError;

pub fn parse_loot(json: &JsonValue) -> Result<LootItems, ParsingError> {
    let items = parse_json_to_loot_item(json)?;

    let mut champion_shards = Vec::new();
    let mut skin_shards = Vec::new();
    let mut mastery_tokens = Vec::new();
    let mut credits = Credits::new();
    let mut ignored = Vec::new();

    for loot_item in items {
        match (
            loot_item.display_category.as_str(),
            loot_item.loot_type.as_str(),
        ) {
            ("CHAMPION", _) => champion_shards.push(parse_champion_shard(loot_item)),
            ("SKIN", _) => skin_shards.push(parse_skin_shard(loot_item)),
            ("CHEST", "CHAMPION_TOKEN") => mastery_tokens.push(parse_mastery_token(loot_item)),
            (_, "CURRENCY") => {
                let value = loot_item.count;
                match loot_item.loot_name.as_str() {
                    "CURRENCY_champion" => credits.blue_essence = value,
                    "CURRENCY_cosmetic" => credits.orange_essence = value,
                    "CURRENCY_mythic" => credits.mythic_essence = value,
                    "CURRENCY_RP" => credits.riot_points = value,
                    _ => ignored.push(loot_item),
                }
            }
            _ => ignored.push(loot_item),
        }
    }

    Ok(LootItems {
        mastery_tokens,
        champion_shards,
        skin_shards,
        credits,
        ignored,
    })
}

fn parse_json_to_loot_item(json: &JsonValue) -> Result<Vec<JsonLootItem>, ParsingError> {
    if let JsonValue::Array(array) = json {
        let mut items = Vec::new();

        for item in array {
            if let JsonValue::Object(item_obj) = &item {
                let display_category = item_obj["displayCategories"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("displayCategories".into()))?;
                let loot_type = item_obj["type"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("type".into()))?;
                let count = item_obj["count"]
                    .as_u32()
                    .ok_or(ParsingError::InvalidType("count".into()))?;
                let ref_id = item_obj["refId"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("refId".into()))?;
                let store_item_id = item_obj["storeItemId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("storeItemId".into()))?;
                let parent_store_item_id = item_obj["parentStoreItemId"]
                    .as_i32()
                    .ok_or(ParsingError::InvalidType("parentStoreItemId".into()))?;
                let loot_name = item_obj["lootName"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("lootName".into()))?;
                let item_desc = item_obj["itemDesc"]
                    .as_str()
                    .ok_or(ParsingError::InvalidType("itemDesc".into()))?;
                let disenchant_value = item_obj["disenchantValue"]
                    .as_u16()
                    .ok_or(ParsingError::InvalidType("disenchantValue".into()))?;

                items.push(JsonLootItem {
                    display_category: display_category.to_string(),
                    loot_type: loot_type.to_string(),
                    count,
                    ref_id: ref_id.to_string(),
                    store_item_id,
                    parent_store_item_id,
                    loot_name: loot_name.to_string(),
                    item_desc: item_desc.to_string(),
                    disenchant_value,
                })
            } else {
                return Err(ParsingError::InvalidType("item".into()));
            }
        }

        return Ok(items);
    }

    Err(ParsingError::InvalidType("items".into()))
}

fn parse_champion_shard(json_item: JsonLootItem) -> ChampionShard {
    ChampionShard {
        champ_id: json_item.store_item_id.into(),
        count: json_item.count as u8,
        disenchant_value: json_item.disenchant_value,
    }
}

fn parse_skin_shard(json_item: JsonLootItem) -> SkinShard {
    SkinShard {
        skin_id: json_item.store_item_id.into(),
        count: json_item.count as u8,
    }
}

fn parse_mastery_token(json_item: JsonLootItem) -> MasteryToken {
    MasteryToken {
        champ_id: json_item.ref_id.into(),
        count: json_item.count as u8,
        level: if json_item.loot_name == "CHAMPION_TOKEN_7" {
            7
        } else {
            6
        },
    }
}
