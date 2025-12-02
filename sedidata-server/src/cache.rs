use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::RwLock;

use crate::model::{ChampionMastery, LeagueEntry};

const CACHE_FILE: &str = "cache.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuuidCacheEntry {
    pub puuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDataCacheEntry {
    pub level: u64,
    pub ranked_stats: Vec<LeagueEntry>,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionMasteryCacheEntry {
    pub mastery: ChampionMastery,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    #[serde(serialize_with = "serialize_tuple_map", deserialize_with = "deserialize_tuple_map")]
    puuid_cache: HashMap<(String, String), PuuidCacheEntry>,

    player_data_cache: HashMap<String, PlayerDataCacheEntry>,

    #[serde(serialize_with = "serialize_tuple_map", deserialize_with = "deserialize_tuple_map")]
    champion_mastery_cache: HashMap<(String, String), ChampionMasteryCacheEntry>,
}

#[derive(Serialize, Deserialize)]
struct Pair {
    first: String,
    second: String,
}

#[derive(Serialize, Deserialize)]
struct TupleEntry<V> {
    key: Pair,
    value: V,
}

fn serialize_tuple_map<V, S>(map: &HashMap<(String, String), V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    V: Serialize,
{
    let entries: Vec<TupleEntry<&V>> = map
        .iter()
        .map(|((first, second), value)| TupleEntry {
            key: Pair {
                first: first.clone(),
                second: second.clone(),
            },
            value,
        })
        .collect();

    entries.serialize(serializer)
}

fn deserialize_tuple_map<'de, V, D>(deserializer: D) -> Result<HashMap<(String, String), V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    let entries: Vec<TupleEntry<V>> = Vec::deserialize(deserializer)?;
    Ok(entries
        .into_iter()
        .map(|entry| ((entry.key.first, entry.key.second), entry.value))
        .collect())
}

pub struct Cache {
    puuid_cache: Arc<RwLock<HashMap<(String, String), PuuidCacheEntry>>>,
    player_data_cache: Arc<RwLock<HashMap<String, PlayerDataCacheEntry>>>,
    champion_mastery_cache: Arc<RwLock<HashMap<(String, String), ChampionMasteryCacheEntry>>>,
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Self {
            puuid_cache: Arc::clone(&self.puuid_cache),
            player_data_cache: Arc::clone(&self.player_data_cache),
            champion_mastery_cache: Arc::clone(&self.champion_mastery_cache),
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        let (puuid_cache, player_data_cache, champion_mastery_cache) = Self::load_from_disk();

        Self {
            puuid_cache: Arc::new(RwLock::new(puuid_cache)),
            player_data_cache: Arc::new(RwLock::new(player_data_cache)),
            champion_mastery_cache: Arc::new(RwLock::new(champion_mastery_cache)),
        }
    }

    #[allow(clippy::type_complexity)]
    fn load_from_disk() -> (
        HashMap<(String, String), PuuidCacheEntry>,
        HashMap<String, PlayerDataCacheEntry>,
        HashMap<(String, String), ChampionMasteryCacheEntry>,
    ) {
        if !Path::new(CACHE_FILE).exists() {
            println!("No cache file found, starting with empty cache");
            return (HashMap::new(), HashMap::new(), HashMap::new());
        }

        match std::fs::read_to_string(CACHE_FILE) {
            Ok(contents) => match serde_json::from_str::<CacheData>(&contents) {
                Ok(data) => {
                    println!(
                        "Loaded cache: {} PUUIDs, {} player data entries, {} champion mastery entries",
                        data.puuid_cache.len(),
                        data.player_data_cache.len(),
                        data.champion_mastery_cache.len()
                    );
                    (data.puuid_cache, data.player_data_cache, data.champion_mastery_cache)
                }
                Err(e) => {
                    eprintln!("Failed to parse cache file: {}", e);
                    (HashMap::new(), HashMap::new(), HashMap::new())
                }
            },
            Err(e) => {
                eprintln!("Failed to read cache file: {}", e);
                (HashMap::new(), HashMap::new(), HashMap::new())
            }
        }
    }

    async fn save_to_disk(&self) {
        let puuid_cache = self.puuid_cache.read().await.clone();
        let player_data_cache = self.player_data_cache.read().await.clone();
        let champion_mastery_cache = self.champion_mastery_cache.read().await.clone();

        let data = CacheData {
            puuid_cache,
            player_data_cache,
            champion_mastery_cache,
        };

        match serde_json::to_string_pretty(&data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(CACHE_FILE, json) {
                    eprintln!("Failed to write cache to disk: {}", e);
                } else {
                    println!("  Cache saved to disk");
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize cache: {}", e);
            }
        }
    }

    pub async fn get_puuid(&self, name: &str, tagline: &str) -> Option<String> {
        let cache = self.puuid_cache.read().await;
        cache
            .get(&(name.to_string(), tagline.to_string()))
            .map(|entry| entry.puuid.clone())
    }

    pub async fn store_puuid(&self, name: String, tagline: String, puuid: String) {
        let mut cache = self.puuid_cache.write().await;
        cache.insert((name, tagline), PuuidCacheEntry { puuid });
        drop(cache);
        self.save_to_disk().await;
    }

    pub async fn get_player_data(&self, puuid: &str) -> Option<PlayerDataCacheEntry> {
        let cache = self.player_data_cache.read().await;
        let entry = cache.get(puuid)?;

        // Check if cached in the last hour
        let now = Utc::now();
        let age = now.signed_duration_since(entry.cached_at);

        (age < Duration::hours(1)).then_some(entry.clone())
    }

    pub async fn store_player_data(&self, puuid: String, level: u64, ranked_stats: Vec<LeagueEntry>) {
        let mut cache = self.player_data_cache.write().await;
        cache.insert(
            puuid,
            PlayerDataCacheEntry {
                level,
                ranked_stats,
                cached_at: Utc::now(),
            },
        );
        drop(cache);
        self.save_to_disk().await;
    }

    pub async fn get_champion_mastery(&self, puuid: &str, champion: &str) -> Option<ChampionMasteryCacheEntry> {
        let cache = self.champion_mastery_cache.read().await;
        let entry = cache.get(&(puuid.to_string(), champion.to_string()))?;

        // Check if cached in the last hour
        let now = Utc::now();
        let age = now.signed_duration_since(entry.cached_at);

        (age < Duration::hours(1)).then_some(entry.clone())
    }

    pub async fn store_champion_mastery(&self, puuid: String, champion: String, mastery: ChampionMastery) {
        let mut cache = self.champion_mastery_cache.write().await;
        cache.insert(
            (puuid, champion),
            ChampionMasteryCacheEntry {
                mastery,
                cached_at: Utc::now(),
            },
        );
        drop(cache);
        self.save_to_disk().await;
    }
}
