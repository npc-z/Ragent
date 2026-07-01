use anyhow::Context;
use std::env;

use ragent::{error::RagentError, llm::engine::agent::Engine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 从 .env 文件加载环境变量
    dotenvy::dotenv().context("Failed to load .env file.")?;

    // 读取单个环境变量
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| RagentError::EnvVarMissing("OPENAI_API_KEY".to_string()))?;
    let api_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = env::var("MODEL").map_err(|_| RagentError::EnvVarMissing("MODEL".to_string()))?;

    let mut engine = Engine::new(api_url, api_key, model)?;

    // 响应
    engine.run_loop().await?;

    Ok(())
}
