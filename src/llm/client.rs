use std::time::Duration;

use crate::error::RagentError;
use crate::llm::response::{ParsedResponse, ResponseParser};
use reqwest::Client;
use serde::Serialize;

#[derive(Debug)]
pub struct ApiClient {
    parser: Box<dyn ResponseParser>,
    base_url: String,
    api_key: String,
    timeout_secs: u64,
}

impl ApiClient {
    pub fn builder() -> ApiClientBuilder {
        ApiClientBuilder::default()
    }

    pub async fn send(&self, body: &impl Serialize) -> Result<ParsedResponse, RagentError> {
        let client = Client::new();

        let raw_resp = client
            .post(self.base_url.clone())
            // TODO: api key format
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| RagentError::ApiRequest(e.to_string()))?;

        let status = raw_resp.status();
        let text = raw_resp
            .text()
            .await
            .map_err(|e| RagentError::ApiRequest(e.to_string()))?;

        // println!(
        //     "the api request body: {}",
        //     serde_json::to_string_pretty(body).expect("invalid body")
        // );
        // println!("the api response status: {}, text is: {}", status, text);

        if !status.is_success() {
            return Err(RagentError::ApiStatus {
                status: status.as_u16(),
                body: text,
            });
        }

        // FIXME: hardcode `llm`
        self.parser.parse(&text, "llm")
    }
}

#[derive(Default)]
pub struct ApiClientBuilder {
    parser: Option<Box<dyn ResponseParser>>,
    base_url: Option<String>,
    api_key: Option<String>,
    headers: Vec<(String, String)>,
    timeout_secs: Option<u64>,
}

impl ApiClientBuilder {
    /// set parser
    pub fn parser(mut self, parser: Box<dyn ResponseParser>) -> Self {
        self.parser = Some(parser);
        self
    }

    // set base_url
    pub fn set_base_url(mut self, url: String) -> Self {
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
    pub fn build(self) -> Result<ApiClient, RagentError> {
        Ok(ApiClient {
            parser: self
                .parser
                .ok_or(RagentError::ClientBuild("parser is required".to_string()))?,

            base_url: self
                .base_url
                .ok_or(RagentError::ClientBuild("base_url is required".to_string()))?,

            api_key: self
                .api_key
                .ok_or(RagentError::ClientBuild("api_key is required".to_string()))?,

            timeout_secs: self.timeout_secs.unwrap_or(30),
        })
    }
}
