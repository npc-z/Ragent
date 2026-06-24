use std::path::PathBuf;

use serde_json::{Value, json};

use crate::llm::deepseek::enums::finish_reason::FinishReason;
use crate::llm::deepseek::enums::tool_call_type::ToolCallType;
use crate::llm::response::{ApiResponse, ResponseMessage};
use crate::llm::{client::ApiClient, llm_type::LlmType};
use crate::tool_call::bash::BashFunction;
use crate::tool_call::edit_file::EditFileFunction;
use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::glob_file::GlobFileFunction;
use crate::tool_call::read_file::ReadFileFunction;
use crate::tool_call::tool::{FunctionTool, ToolCall, ToolResult};
use crate::tool_call::write_file::WriteFileFunction;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Engine {
    client: ApiClient,
    // model: String,
    body: Body,

    /// 对话轮次
    turn_count: u16,
    work_dir: PathBuf,
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
#[serde(tag = "role")]
enum Message {
    #[serde(rename = "system")]
    System { content: String },

    #[serde(rename = "assistant")]
    Assistant {
        content: String,
        reasoning_content: String,
        tool_calls: Option<Vec<ToolCall>>,
    },

    #[serde(rename = "user")]
    User { content: String },

    #[serde(rename = "tool")]
    Tool {
        content: String,
        tool_call_id: String,
    },
}

impl Engine {
    /// New engine
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        let client = ApiClient::builder()
            .llm_type(LlmType::DeepSeek.as_str())
            .base_url(api_url)
            .api_key(api_key)
            .timeout(60)
            .build()
            .expect("config ApiClient failed");

        // let cwd = env::current_dir()?;
        // let canonical_cwd = fs::canonicalize(&cwd)?;

        // get current dir
        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            // Err(e) => return format!("Error: failed to get current dir: {}", e),
            Err(e) => panic!("Error: failed to get current dir: {}", e),
        };

        let mut e = Engine {
            client,
            // model: model.clone(),
            body: Body {
                model: model.clone(),
                messages: Vec::new(),
                thinking: json!({"type": "enabled"}),
                reasoning_effort: "high".to_string(),
                stream: false,
                tools: Vec::new(),
            },
            turn_count: 0,
            work_dir: cwd,
        };
        e.init_body();
        e
    }

    /// init the request body
    fn init_body(&mut self) {
        self.init_messages();
        self.init_tools();
    }

    fn add_message(&mut self, msg: Message) {
        self.show_message(&msg);
        self.body.messages.push(msg);
    }

    fn show_message(&self, message: &Message) {
        let begin = ">".repeat(40);
        let end = "<".repeat(40);

        match message {
            Message::System { content } => {
                println!("{}\nsystem:\n{}\n{}\n\n", begin, content, end)
            }

            Message::Assistant {
                content,
                reasoning_content,
                ..
            } => println!(
                "{}\nreasoning_content:\n{}\nassistant:\n{}\n{}\n\n",
                begin, reasoning_content, content, end
            ),

            Message::User { content, .. } => {
                println!("{}\nuser:\n{}\n{}\n\n", begin, content, end)
            }

            Message::Tool {
                content: _content,
                tool_call_id,
            } => {
                println!(
                    "{}\ntool({}):\n{}\n{}\n\n",
                    begin, tool_call_id, "muted now", end
                )
            }
        }
    }

    /// add user message to the body
    fn add_user_message(&mut self, content: String) {
        self.add_message(Message::User { content });
    }

    /// add system message to the body
    fn add_system_message(&mut self, content: String) {
        self.add_message(Message::System { content });
    }

    /// add tool call result message to the body
    fn add_tool_call_result_message(&mut self, tool_result: ToolResult) {
        self.add_message(Message::Tool {
            content: tool_result.content,
            tool_call_id: tool_result.tool_use_id,
        });
    }

    /// add llm response message to the body
    fn add_llm_response_message(&mut self, msg: ResponseMessage) {
        self.add_message(Message::Assistant {
            content: msg.content,
            reasoning_content: msg.reasoning_content,
            tool_calls: msg.tool_calls,
        })
    }

    /// init the system message
    fn init_messages(&mut self) {
        let promotion = format!(
            "You are a coding agent at {}. Use tools to solve tasks. Act first, then report clearly",
            self.work_dir.display()
        );
        self.add_system_message(promotion.to_string());
    }

    /// init tools, currently only bash
    fn init_tools(&mut self) {
        // bash
        self.body.tools.push(json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::Bash.as_str(),
                "description": "Run a shell command.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string",},
                    },
                    "required": ["command"],
                },
            },
        }));

        // read file
        self.body.tools.push(json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::ReadFile.as_str(),
                "description": "Read file contents.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "limit": {"type": "integer",},
                    },
                    "required": ["path"],
                },
            },
        }));

        // write file
        self.body.tools.push(json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::WriteFile.as_str(),
                "description": "Write content to a file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "content": {"type": "string",},
                    },
                    "required": ["path", "content"],
                },
            },
        }));

        // edit file
        self.body.tools.push(json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::EditFile.as_str(),
                "description": "Replace exact text in a file once",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "old_text": {"type": "string",},
                        "new_text": {"type": "string",},
                    },
                    "required": ["path", "old_text", "new_text"],
                },
            },
        }));

        // glob
        self.body.tools.push(json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::Glob.as_str(),
                "description": "Find files matching a glob pattern",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "pattern": {"type": "string",},
                    },
                    "required": ["path","pattern"],
                },
            },
        }));
    }

    /// Send message to the LLM and get the response
    async fn send_message(&self) -> impl ApiResponse + use<'_> {
        self.client.send(&self.body).await
    }

    /// Read user input
    fn get_user_message(&mut self) {
        let user_msg = read_user_input();
        self.add_user_message(user_msg.unwrap_or_default());
    }

    /// Run the message loop, read user input, send message and get the response
    pub async fn run_loop(&mut self) {
        self.get_user_message();
        self.run_one_turn().await;
    }

    /// 进行一个轮次
    async fn run_one_turn(&mut self) {
        let finish_reason;
        let answer;
        let tcqs;
        {
            let response = self.send_message().await;
            finish_reason = response.get_finishi_reason();
            answer = response.get_response_message();
            tcqs = response.get_tool_calls();
        }
        self.add_llm_response_message(answer);

        self.turn_count += 1;

        match finish_reason {
            FinishReason::ToolCalls => {
                self.dyr_run_tool(&tcqs);
                let tool_result = self.run_tools(&tcqs);

                if !tool_result.is_empty() {
                    for tr in &tool_result {
                        self.add_tool_call_result_message(tr.clone());
                    }
                    Box::pin(self.run_one_turn()).await;
                }
            }
            FinishReason::Stop => {}
        }
    }

    fn run_tools(&self, tool_calls: &Vec<ToolCall>) -> Vec<ToolResult> {
        let mut r: Vec<ToolResult> = Vec::new();

        for tc in tool_calls {
            match tc.r#type {
                ToolCallType::Function => match tc.function.name {
                    // bash
                    ToolFunctionType::Bash => {
                        let func = BashFunction::new(tc.id.clone(), tc.function.arguments.clone());
                        let call_result = func.run();
                        r.push(call_result);
                    }

                    // read file
                    ToolFunctionType::ReadFile => {
                        let func = ReadFileFunction::new(
                            self.work_dir.clone(),
                            tc.id.clone(),
                            tc.function.arguments.clone(),
                        );
                        let call_result = func.run();
                        r.push(call_result);
                    }

                    // write file
                    ToolFunctionType::WriteFile => {
                        let func = WriteFileFunction::new(
                            self.work_dir.clone(),
                            tc.id.clone(),
                            tc.function.arguments.clone(),
                        );
                        let call_result = func.run();
                        r.push(call_result);
                    }

                    // edit file
                    ToolFunctionType::EditFile => {
                        let func = EditFileFunction::new(
                            self.work_dir.clone(),
                            tc.id.clone(),
                            tc.function.arguments.clone(),
                        );
                        let call_result = func.run();
                        r.push(call_result);
                    }

                    // glob file
                    ToolFunctionType::Glob => {
                        let func = GlobFileFunction::new(
                            self.work_dir.clone(),
                            tc.id.clone(),
                            tc.function.arguments.clone(),
                        );
                        let call_result = func.run();
                        r.push(call_result);
                    }
                },
            }
        }

        r
    }

    fn dyr_run_tool(&self, tool_calls: &Vec<ToolCall>) {
        for tc in tool_calls {
            println!(
                "the idx={} {} {} arguments: {}",
                tc.index,
                tc.function.name.as_str(),
                tc.r#type.as_str(),
                tc.function.arguments
            );
        }
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
