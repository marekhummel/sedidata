use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AccountRequest {
    pub name: String,
    pub tagline: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotAccountResponse {
    pub puuid: String,

    #[allow(dead_code)]
    pub game_name: String,
    #[allow(dead_code)]
    pub tag_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueEntry {
    pub league_id: String,
    pub puuid: String,
    pub queue_type: String,
    pub tier: String,
    pub rank: String,
    pub league_points: i32,
    pub wins: i32,
    pub losses: i32,
    pub hot_streak: bool,
    pub veteran: bool,
    pub fresh_blood: bool,
    pub inactive: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
