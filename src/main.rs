use std::env;

use ragent::llm::engine::engine::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 .env 文件加载环境变量
    dotenvy::dotenv().ok();

    // 读取单个环境变量，如果不存在就 panic（程序立即终止并给出提示）
    let api_key = env::var("OPENAI_API_KEY")
        .expect("请设置环境变量 OPENAI_API_KEY，或在项目根目录创建 .env 文件");
    let api_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    let model = env::var("MODEL").expect("请设置模型名称");
    let mut engine = Engine::new(api_url, api_key, model);

    // 响应
    engine.run_loop().await;

    Ok(())
}
