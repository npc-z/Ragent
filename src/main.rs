use std::env;

use ragent::llm::client::ApiClient;

fn read_user_input() -> Option<String> {
    let mut rl = rustyline::DefaultEditor::new().expect("init input editor failed");
    let readline = rl.readline("ragent> ");

    match readline {
        Ok(line) => {
            let line = line.trim().to_string();
            Some(line)
        }
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 .env 文件加载环境变量
    dotenvy::dotenv().ok();

    // 读取单个环境变量，如果不存在就 panic（程序立即终止并给出提示）
    let api_key = env::var("OPENAI_API_KEY")
        .expect("请设置环境变量 OPENAI_API_KEY，或在项目根目录创建 .env 文件");
    let api_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    let client = ApiClient::builder()
        .base_url(api_url)
        .api_key(api_key)
        .build()
        .expect("config ApiClient failed");

    let model = env::var("MODEL").expect("请设置模型名称");

    let user_msg = read_user_input();

    // 构造 JSON body（也可以用 serde 序列化结构体）
    let system = "You are a coding agent at {os.getcwd()}. Use bash to inspect and change the workspace. Act first, then report clearly";
    let body = serde_json::json!({
        "model": model,
        "messages": [
          {"role": "system", "content": system},
          {"role": "user", "content": user_msg}
        ],
        "thinking": {"type": "enabled"},
        "reasoning_effort": "high",
        "stream": false,
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "bash",
                    "description": "Run a shell command in the current workspace.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {"type": "string",},
                        },
                        "required": ["command"],
                    },
                },
            },
        ],
    });

    // // 读取响应文本
    let response = client.send(&body).await.expect("request failed");
    println!("{}", response.text().await?);

    Ok(())
}
