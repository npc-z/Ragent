use std::time::Duration;

use reqwest::{Client, Error, Response};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    api_key: String,
    timeout_secs: u64,
}

impl ApiClient {
    pub fn builder() -> ApiClientBuilder {
        ApiClientBuilder::default()
    }

    pub async fn send(&self, body: &Value) -> Result<Response, Error> {
        let client = Client::new();

        client
            .post(self.base_url.clone())
            // TODO: api key format
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
        // .expect("client: todo the err")
    }
}

#[derive(Default)]
pub struct ApiClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    headers: Vec<(String, String)>,
    timeout_secs: Option<u64>,
}

impl ApiClientBuilder {
    // set base_url
    pub fn base_url(mut self, url: String) -> Self {
        self.base_url = Some(url);
        self
    }

    // set api_key
    pub fn api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    // add single header
    pub fn header(mut self, name: String, value: String) -> Self {
        self.headers.push((name, value));
        self
    }

    // set timeout seconds
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    // build ApiClient
    pub fn build(self) -> Result<ApiClient, &'static str> {
        Ok(ApiClient {
            base_url: self.base_url.ok_or("base_url is required")?,
            api_key: self.api_key.ok_or("api_key is required")?,
            timeout_secs: self.timeout_secs.unwrap_or(30),
        })
    }
}
