use std::time::Duration;

use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct GitHubClient {
    http: Client,
    token: String,
}

impl GitHubClient {
    pub fn from_env() -> Result<Self, String> {
        let token = std::env::var("GITHUB_TOKEN")
            .map_err(|_| "Missing GITHUB_TOKEN in environment (.env)".to_string())?;
        Ok(Self::new(token))
    }

    pub fn new(token: String) -> Self {
        let http = Client::builder()
            .user_agent("gh-pr-tui")
            .timeout(Duration::from_secs(20))
            .build()
            .expect("reqwest client build");
        Self { http, token }
    }

    pub fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, String> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        let resp = self
            .http
            .post("https://api.github.com/graphql")
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .map_err(|e| format!("GitHub request failed: {e}"))?;

        let status = resp.status();
        let text = resp.text().map_err(|e| format!("Read body failed: {e}"))?;

        if !status.is_success() {
            return Err(format!("GitHub HTTP {status}: {text}"));
        }

        serde_json::from_str(&text).map_err(|e| format!("JSON parse failed: {e} (body={text})"))
    }
}