use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::RwLock;

use crate::model::LeagueEntry;

const CACHE_FILE: &str = "cache.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuuidCache {
    pub puuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDataCache {
    pub level: u64,
    pub ranked_stats: Vec<LeagueEntry>,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    #[serde(serialize_with = "serialize_tuple_map", deserialize_with = "deserialize_tuple_map")]
    puuid_cache: HashMap<(String, String), PuuidCache>,
    player_data_cache: HashMap<String, PlayerDataCache>,
}

fn serialize_tuple_map<S>(map: &HashMap<(String, String), PuuidCache>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut seq = serializer.serialize_map(Some(map.len()))?;
    for ((name, tag), value) in map {
        let key = format!("{}#{}", name, tag);
        seq.serialize_entry(&key, value)?;
    }
    seq.end()
}

fn deserialize_tuple_map<'de, D>(deserializer: D) -> Result<HashMap<(String, String), PuuidCache>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: HashMap<String, PuuidCache> = HashMap::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .filter_map(|(key, value)| {
            let parts: Vec<&str> = key.split('#').collect();
            if parts.len() == 2 {
                Some(((parts[0].to_string(), parts[1].to_string()), value))
            } else {
                None
            }
        })
        .collect())
}

pub struct Cache {
    puuid_cache: Arc<RwLock<HashMap<(String, String), PuuidCache>>>,
    player_data_cache: Arc<RwLock<HashMap<String, PlayerDataCache>>>,
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Self {
            puuid_cache: Arc::clone(&self.puuid_cache),
            player_data_cache: Arc::clone(&self.player_data_cache),
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        let (puuid_cache, player_data_cache) = Self::load_from_disk();

        Self {
            puuid_cache: Arc::new(RwLock::new(puuid_cache)),
            player_data_cache: Arc::new(RwLock::new(player_data_cache)),
        }
    }

    fn load_from_disk() -> (HashMap<(String, String), PuuidCache>, HashMap<String, PlayerDataCache>) {
        if !Path::new(CACHE_FILE).exists() {
            println!("No cache file found, starting with empty cache");
            return (HashMap::new(), HashMap::new());
        }

        match std::fs::read_to_string(CACHE_FILE) {
            Ok(contents) => match serde_json::from_str::<CacheData>(&contents) {
                Ok(data) => {
                    println!(
                        "Loaded cache: {} PUUIDs, {} player data entries",
                        data.puuid_cache.len(),
                        data.player_data_cache.len()
                    );
                    (data.puuid_cache, data.player_data_cache)
                }
                Err(e) => {
                    eprintln!("Failed to parse cache file: {}", e);
                    (HashMap::new(), HashMap::new())
                }
            },
            Err(e) => {
                eprintln!("Failed to read cache file: {}", e);
                (HashMap::new(), HashMap::new())
            }
        }
    }

    async fn save_to_disk(&self) {
        let puuid_cache = self.puuid_cache.read().await.clone();
        let player_data_cache = self.player_data_cache.read().await.clone();

        let data = CacheData {
            puuid_cache,
            player_data_cache,
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
        cache.insert((name, tagline), PuuidCache { puuid });
        drop(cache);
        self.save_to_disk().await;
    }

    pub async fn get_player_data(&self, puuid: &str) -> Option<PlayerDataCache> {
        let cache = self.player_data_cache.read().await;
        let entry = cache.get(puuid)?;

        // Check if cached today
        let now = Utc::now();
        let age = now.signed_duration_since(entry.cached_at);

        (age < Duration::hours(1)).then_some(entry.clone())
    }

    pub async fn store_player_data(&self, puuid: String, level: u64, ranked_stats: Vec<LeagueEntry>) {
        let mut cache = self.player_data_cache.write().await;
        cache.insert(
            puuid,
            PlayerDataCache {
                level,
                ranked_stats,
                cached_at: Utc::now(),
            },
        );
        drop(cache);
        self.save_to_disk().await;
    }
}
