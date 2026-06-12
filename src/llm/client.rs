use std::str::FromStr;
use std::time::Duration;

use reqwest::Client;
use serde_json::Value;

use crate::llm::deepseek::response::DeepseekResponse;
use crate::llm::{llm_type::LlmType, response::ApiResponse};

#[derive(Debug)]
pub struct ApiClient {
    llm: LlmType,
    base_url: String,
    api_key: String,
    timeout_secs: u64,
}

impl ApiClient {
    pub fn builder() -> ApiClientBuilder {
        ApiClientBuilder::default()
    }

    pub async fn send(&self, body: &Value) -> impl ApiResponse {
        let client = Client::new();

        let raw_resp = client
            .post(self.base_url.clone())
            // TODO: api key format
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .expect("client: todo the err");

        let text = raw_resp.text().await.expect("Unable to read response text");
        println!("the api text is: {}", text);

        match self.llm {
            LlmType::DeepSeek => serde_json::from_str::<DeepseekResponse>(text.as_str())
                .unwrap_or_else(|e| {
                    panic!(
                        "Unable to transform {} api response, the err is {}",
                        LlmType::DeepSeek,
                        e
                    )
                }),
        }
    }
}

#[derive(Default)]
pub struct ApiClientBuilder {
    llm: Option<LlmType>,
    base_url: Option<String>,
    api_key: Option<String>,
    headers: Vec<(String, String)>,
    timeout_secs: Option<u64>,
}

impl ApiClientBuilder {
    // set llm type
    pub fn llm_type(mut self, llm_type: &str) -> Self {
        let llm =
            LlmType::from_str(llm_type).unwrap_or_else(|_| panic!("Unsupported llm {}", llm_type));
        self.llm = Some(llm);
        self
    }
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
            llm: self.llm.ok_or("llm is required")?,
            base_url: self.base_url.ok_or("base_url is required")?,
            api_key: self.api_key.ok_or("api_key is required")?,
            timeout_secs: self.timeout_secs.unwrap_or(30),
        })
    }
}
