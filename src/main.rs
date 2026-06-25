use anyhow::Context;
use std::env;

use ragent::llm::engine::engine::Engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 从 .env 文件加载环境变量
    dotenvy::dotenv().context("Failed to load .env file.")?;

    // 读取单个环境变量
    let api_key = env::var("OPENAI_API_KEY").context(
        "Did not find OPENAI_API_KEY in environment variables. Please set it in your .env file.",
    )?;
    let api_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    let model = env::var("MODEL")
        .context("Did not find MODEL in environment variables. Please set it in your .env file.")?;
    let mut engine = Engine::new(api_url, api_key, model)?;

    // 响应
    engine.run_loop().await?;

    Ok(())
}
