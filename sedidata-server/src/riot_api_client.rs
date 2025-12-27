use reqwest::{Client, Response};
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct RiotApiClient {
    client: Client,
    api_key: String,
}

impl RiotApiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        const MAX_RETRIES: usize = 5;
        const RETRY_DELAYS: [u64; 5] = [1, 2, 3, 5, 8];

        let mut attempts = 0;
        loop {
            let resp = self
                .client
                .get(url)
                .header("X-Riot-Token", &self.api_key)
                .send()
                .await?;

            if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS && attempts < MAX_RETRIES {
                attempts += 1;
                sleep(Duration::from_secs(RETRY_DELAYS[attempts - 1])).await;
                continue;
            }

            return Ok(resp);
        }
    }
}
