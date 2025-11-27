use std::env;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

use reqwest::Response;
use tower_http::cors::{Any, CorsLayer};

mod cache;
mod model;

use cache::Cache;
use model::*;

#[derive(Clone)]
struct AppState {
    api_key: String,
    cache: Cache,
}

#[tokio::main]
async fn main() {
    // Load API key from environment variable
    let api_key = env::var("RIOT_API_KEY").expect("RIOT_API_KEY environment variable must be set");

    // Initialize cache
    let cache = Cache::new();

    // Create app state
    let state = AppState { api_key, cache };
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    // Build router
    let app = Router::new()
        .route("/league", get(get_league_entries))
        .route("/heartbeat", get(heartbeat))
        .layer(cors)
        .with_state(state);

    // Determine port from environment or use default
    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("Sedidata server running on http://{}", addr);
    axum::serve(listener, app).await.expect("Server failed");
}

async fn get_league_entries(Query(params): Query<AccountRequest>, State(state): State<AppState>) -> impl IntoResponse {
    println!("Request received for Riot ID: {}#{}", params.name, params.tagline);

    let client = reqwest::Client::new();

    // Check PUUID cache first
    let puuid = if let Some(cached_puuid) = state.cache.get_puuid(&params.name, &params.tagline).await {
        println!("  PUUID found in cache");
        cached_puuid
    } else {
        println!("  Fetching PUUID from Riot API");
        let fetched_puuid = match request_puuid(&params.name, &params.tagline, &state, &client).await {
            Ok(puuid) => puuid,
            Err(resp) => return resp,
        };

        // Cache the PUUID permanently
        println!("  PUUID cached");
        state
            .cache
            .store_puuid(params.name.clone(), params.tagline.clone(), fetched_puuid.clone())
            .await;
        fetched_puuid
    };

    // Check player data cache
    if let Some(cached_data) = state.cache.get_player_data(&puuid).await {
        println!("  Player data found in cache (from today)");
        let entries_json = serde_json::to_string(&cached_data.ranked_stats).unwrap_or_else(|_| "[]".to_string());
        let combined_json = format!(r#"{{"level":{},"ranked_stats":{}}}"#, cached_data.level, entries_json);
        return (StatusCode::OK, combined_json).into_response();
    }

    println!("  Fetching fresh player data from Riot API");
    let (entries, level) = match request_player_data(&puuid, &state, &client).await {
        Ok(data) => data,
        Err(resp) => return resp,
    };

    // Cache the player data
    println!("  Player data cached");
    state.cache.store_player_data(puuid, level, entries.clone()).await;

    // Combine into response JSON
    let entries_json = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
    let combined_json = format!(r#"{{"level":{},"ranked_stats":{}}}"#, level, entries_json);

    (StatusCode::OK, combined_json).into_response()
}

async fn request_puuid(
    name: &str,
    tagline: &str,
    state: &AppState,
    client: &reqwest::Client,
) -> Result<String, axum::response::Response> {
    // Step 1: Get PUUID from Riot ID
    let account_url = format!(
        "https://europe.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
        name, tagline
    );

    // Request account info to get PUUID
    let account_response = client
        .get(&account_url)
        .header("X-Riot-Token", &state.api_key)
        .send()
        .await;

    match account_response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<RiotAccountResponse>().await {
                    Ok(account) => Ok(account.puuid),
                    Err(e) => {
                        eprintln!("Failed to parse account response: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to parse account response from Riot API".to_string(),
                            }),
                        )
                            .into_response())
                    }
                }
            } else {
                let status = resp.status();
                eprintln!("Riot API returned error for account lookup: {}", status);
                Err((
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Riot API returned error for account lookup: {}", status),
                    }),
                )
                    .into_response())
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to Riot API for account lookup: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to connect to Riot API for account lookup".to_string(),
                }),
            )
                .into_response())
        }
    }
}

async fn request_player_data(
    puuid: &str,
    state: &AppState,
    client: &reqwest::Client,
) -> Result<(Vec<LeagueEntry>, u64), axum::response::Response> {
    let league_url = format!(
        "https://euw1.api.riotgames.com/lol/league/v4/entries/by-puuid/{}",
        puuid
    );

    let summoner_url = format!(
        "https://euw1.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/{}",
        puuid
    );

    // Make both requests in parallel
    let league_future = client.get(&league_url).header("X-Riot-Token", &state.api_key).send();
    let summoner_future = client.get(&summoner_url).header("X-Riot-Token", &state.api_key).send();

    let (league_response, summoner_response) = tokio::join!(league_future, summoner_future);

    let entries = extract_league_entries(league_response).await?;
    let level = extract_level(summoner_response).await?;
    Ok((entries, level))
}

async fn extract_level(response: Result<Response, reqwest::Error>) -> Result<u64, axum::response::Response> {
    // Placeholder function if needed in future
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text().await {
                    Ok(text) => {
                        // Parse JSON to extract level
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(json) => Ok(json["summonerLevel"].as_u64().unwrap_or(0)),
                            Err(e) => {
                                eprintln!("Failed to parse summoner response: {}", e);
                                Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(ErrorResponse {
                                        error: "Failed to parse summoner response from Riot API".to_string(),
                                    }),
                                )
                                    .into_response())
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read summoner response: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to read summoner response".to_string(),
                            }),
                        )
                            .into_response())
                    }
                }
            } else {
                let status = resp.status();
                eprintln!("Riot API returned error for summoner lookup: {}", status);
                Err((
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Riot API returned error for summoner lookup: {}", status),
                    }),
                )
                    .into_response())
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to Riot API for summoner lookup: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to connect to Riot API for summoner lookup".to_string(),
                }),
            )
                .into_response())
        }
    }
}

async fn extract_league_entries(
    response: Result<Response, reqwest::Error>,
) -> Result<Vec<LeagueEntry>, axum::response::Response> {
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<Vec<LeagueEntry>>().await {
                    Ok(entries) => Ok(entries),
                    Err(e) => {
                        eprintln!("Failed to parse league entries response: {}", e);
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to parse league entries from Riot API".to_string(),
                            }),
                        )
                            .into_response())
                    }
                }
            } else {
                let status = resp.status();
                eprintln!("Riot API returned error for league entries: {}", status);
                Err((
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Riot API returned error for league entries: {}", status),
                    }),
                )
                    .into_response())
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to Riot API for league entries: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to connect to Riot API for league entries".to_string(),
                }),
            )
                .into_response())
        }
    }
}

async fn heartbeat() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
