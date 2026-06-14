use std::str::FromStr;

use serde_json::Value;

use crate::llm::deepseek::enums::finish_reason::FinishReason;
use crate::llm::engine::role::Role;
use crate::llm::response::ApiResponse;
use crate::llm::{client::ApiClient, llm_type::LlmType};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Engine {
    client: ApiClient,
    // model: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    model: String,
    messages: Vec<Message>,
    thinking: Value,
    reasoning_effort: String,
    stream: bool,
    tools: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: Role,
    content: String,
}

impl Engine {
    /// New engine
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        let client = ApiClient::builder()
            .llm_type(LlmType::DeepSeek.as_str())
            .base_url(api_url)
            .api_key(api_key)
            .build()
            .expect("config ApiClient failed");

        let mut e = Engine {
            client,
            // model: model.clone(),
            body: Body {
                model: model.clone(),
                messages: Vec::new(),
                thinking: serde_json::json!({"type": "enabled"}),
                reasoning_effort: "high".to_string(),
                stream: false,
                tools: Vec::new(),
            },
        };
        e.init_body();
        e
    }

    /// init the request body
    fn init_body(&mut self) {
        self.init_messages();
        self.init_tools();
    }

    /// add message to the body
    fn add_message(&mut self, role: String, content: String) {
        let m = Message {
            role: Role::from_str(role.as_str()).expect("cant parse the role"),
            content,
        };
        self.body.messages.push(m);
    }

    /// init the system message
    fn init_messages(&mut self) {
        let system = "You are a coding agent at {os.getcwd()}. Use bash to inspect and change the workspace. Act first, then report clearly";
        self.add_message("system".to_string(), system.to_string());
    }

    /// init tools, currently only bash
    fn init_tools(&mut self) {
        self.body.tools.push(serde_json::json!({
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
        }));
    }

    /// Send message to the LLM and get the response
    async fn send_message(&self) -> impl ApiResponse {
        self.client.send(&self.body).await
    }

    /// Read user input
    fn get_user_mesage(&mut self) {
        let user_msg = read_user_input();
        self.add_message("user".to_string(), user_msg.unwrap_or_default());
    }

    /// Run the message loop, read user input, send message and get the response
    pub async fn message_loop(&mut self) {
        self.show_message(self.body.messages[0].clone());
        self.get_user_mesage();
        self.show_message(self.body.messages[1].clone());

        let response = self.send_message().await;
        let finish_reason = &response.get_finishi_reason();
        let answer = response.get_answer();

        self.add_message(Role::User.to_string(), answer);

        match finish_reason {
            FinishReason::ToolCalls => {
                response.dyr_run_tool();
                let tool_result = response.run_tool();
                println!("the tool result is: {}", tool_result);
            }
            FinishReason::Stop => {}
        }
    }

    fn show_message(&mut self, message: Message) {
        println!("{}: {}", message.role, message.content);
    }
}

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
