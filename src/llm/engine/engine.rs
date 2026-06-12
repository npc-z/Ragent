use serde_json::Value;

use crate::llm::response::ApiResponse;
use crate::llm::{client::ApiClient, llm_type::LlmType};

#[derive(Debug)]
pub struct Engine {
    client: ApiClient,
}

impl Engine {
    /// New engine
    pub fn new(api_url: String, api_key: String) -> Self {
        let client = ApiClient::builder()
            .llm_type(LlmType::DeepSeek.as_str())
            .base_url(api_url)
            .api_key(api_key)
            .build()
            .expect("config ApiClient failed");

        Engine { client }
    }

    /// Send message to the llm and get response
    pub async fn message(&self, body: &Value) -> impl ApiResponse {
        self.client.send(body).await
    }
}
