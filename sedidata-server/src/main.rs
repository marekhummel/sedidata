use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

use std::env;
use tower_http::cors::{Any, CorsLayer};

mod model;

use model::*;

#[tokio::main]
async fn main() {
    // Load API key from environment variable
    let api_key = env::var("RIOT_API_KEY").expect("RIOT_API_KEY environment variable must be set");

    // Create app state
    let state = AppState { api_key };
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

#[derive(Clone)]
struct AppState {
    api_key: String,
}

async fn get_league_entries(
    Query(params): Query<AccountRequest>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();

    println!("Request received for Riot ID: {}#{}", params.name, params.tagline);

    // Step 1: Get PUUID from Riot ID
    let account_url = format!(
        "https://europe.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
        params.name, params.tagline
    );

    // Request account info to get PUUID
    let account_response = client
        .get(&account_url)
        .header("X-Riot-Token", &state.api_key)
        .send()
        .await;

    let puuid = match account_response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<RiotAccountResponse>().await {
                    Ok(account) => account.puuid,
                    Err(e) => {
                        eprintln!("Failed to parse account response: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to parse account response from Riot API".to_string(),
                            }),
                        )
                            .into_response();
                    }
                }
            } else {
                let status = resp.status();
                eprintln!("Riot API returned error for account lookup: {}", status);
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Riot API returned error for account lookup: {}", status),
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to Riot API for account lookup: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to connect to Riot API for account lookup".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Step 2: Get league entries and summoner info in parallel
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

    // Process league entries response
    let entries = match league_response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<Vec<LeagueEntry>>().await {
                    Ok(entries) => entries,
                    Err(e) => {
                        eprintln!("Failed to parse league entries response: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to parse league entries from Riot API".to_string(),
                            }),
                        )
                            .into_response();
                    }
                }
            } else {
                let status = resp.status();
                eprintln!("Riot API returned error for league entries: {}", status);
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Riot API returned error for league entries: {}", status),
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to Riot API for league entries: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to connect to Riot API for league entries".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Process summoner response to extract level
    let level = match summoner_response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text().await {
                    Ok(text) => {
                        // Parse JSON to extract level
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(json) => json["summonerLevel"].as_u64().unwrap_or(0),
                            Err(e) => {
                                eprintln!("Failed to parse summoner response: {}", e);
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(ErrorResponse {
                                        error: "Failed to parse summoner response from Riot API".to_string(),
                                    }),
                                )
                                    .into_response();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read summoner response: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to read summoner response".to_string(),
                            }),
                        )
                            .into_response();
                    }
                }
            } else {
                let status = resp.status();
                eprintln!("Riot API returned error for summoner lookup: {}", status);
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("Riot API returned error for summoner lookup: {}", status),
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to Riot API for summoner lookup: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to connect to Riot API for summoner lookup".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Combine into response JSON
    let entries_json = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
    let combined_json = format!(r#"{{"level":{},"ranked_stats":{}}}"#, level, entries_json);

    (StatusCode::OK, combined_json).into_response()
}

async fn heartbeat() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
