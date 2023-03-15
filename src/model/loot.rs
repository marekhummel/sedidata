use super::ids::{ChampionId, SkinId};

#[derive(Debug)]
pub struct JsonLootItem {
    pub display_category: String,
    pub loot_type: String,
    pub count: u32,
    pub ref_id: String,
    pub store_item_id: i32,
    pub parent_store_item_id: i32,
    pub loot_name: String,
    pub item_desc: String,
    pub disenchant_value: u16,
}

#[derive(Debug)]
pub struct LootItems {
    pub mastery_tokens: Vec<MasteryToken>,
    pub champion_shards: Vec<ChampionShard>,
    pub skin_shards: Vec<SkinShard>,
    pub credits: Credits,
    pub ignored: Vec<JsonLootItem>,
}

#[derive(Debug)]
pub struct Chest {}

#[derive(Debug)]
pub struct MasteryToken {
    pub champ_id: ChampionId,
    pub count: u8,
    pub level: u8,
}

#[derive(Debug)]
pub struct ChampionShard {
    pub champ_id: ChampionId,
    pub count: u8,
    pub disenchant_value: u16,
}

#[derive(Debug)]
pub struct SkinShard {
    pub skin_id: SkinId,
    pub count: u8,
}

#[derive(Debug)]
pub struct Credits {
    pub blue_essence: u32,
    pub orange_essence: u32,
    pub mythic_essence: u32,
    pub riot_points: u32,
}

impl Credits {
    pub fn new() -> Self {
        Self {
            blue_essence: 0,
            orange_essence: 0,
            mythic_essence: 0,
            riot_points: 0,
        }
    }
}
