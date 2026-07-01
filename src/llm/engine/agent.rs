use std::path::PathBuf;

use anyhow::Context;
use serde_json::{Value, json};

use crate::llm::client::ApiClient;
use crate::llm::deepseek::enums::tool_call_type::ToolCallType;
use crate::llm::deepseek::parser::DeepseekParser;
use crate::llm::response::{FinishReason, ParsedResponse, ResponseMessage};
use crate::tool_call::bash::BashTool;
use crate::tool_call::edit_file::EditFileTool;
use crate::tool_call::glob_file::GlobFileTool;
use crate::tool_call::read_file::ReadFileTool;
use crate::tool_call::tool::{ToolCall, ToolRegistry, ToolResult};
use crate::tool_call::write_file::WriteFileTool;
use serde::{Deserialize, Serialize};

pub struct Engine {
    client: ApiClient,
    // model: String,
    body: Body,
    /// tool registry
    tool_registry: ToolRegistry,

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
    pub fn new(api_url: String, api_key: String, model: String) -> anyhow::Result<Self> {
        let client = ApiClient::builder()
            .parser(Box::new(DeepseekParser))
            .set_base_url(api_url)
            .api_key(api_key)
            .timeout(60)
            .build()?;

        // get current dir
        let cwd = std::env::current_dir()
            .context("Failed to get current directory")?
            .canonicalize()
            .context("Failed to canonicalize current directory")?;

        let tool_registry = default_tool_registry();

        Ok(Self::with_deps(client, tool_registry, model, cwd))
    }

    pub fn with_deps(
        client: ApiClient,
        tool_registry: ToolRegistry,
        model: String,
        work_dir: PathBuf,
    ) -> Self {
        let mut e = Self {
            client,
            body: Body {
                model,
                messages: Vec::new(),
                thinking: json!({"type": "enabled"}),
                reasoning_effort: "high".to_string(),
                stream: false,
                tools: tool_registry.schemas(),
            },
            tool_registry,
            turn_count: 0,
            work_dir,
        };
        e.init_messages();
        e
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

    /// Send message to the LLM and get the response
    async fn send_message(&self) -> anyhow::Result<ParsedResponse> {
        Ok(self.client.send(&self.body).await?)
    }

    /// Read user input
    fn get_user_message(&mut self) -> anyhow::Result<()> {
        let user_msg = read_user_input()?;
        self.add_user_message(user_msg);
        Ok(())
    }

    /// Run the message loop, read user input, send message and get the response
    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        self.get_user_message()?;

        // 进行一个轮次
        loop {
            let response = self.send_message().await?;
            let finish_reason = response.finish_reason;
            let answer = response.message;
            let tcqs = response.tool_calls;

            self.add_llm_response_message(answer);
            self.turn_count += 1;

            match finish_reason {
                FinishReason::ToolCalls => {
                    self.dyr_run_tool(&tcqs);
                    let tool_result = self.run_tools(&tcqs)?;

                    if !tool_result.is_empty() {
                        for tr in &tool_result {
                            self.add_tool_call_result_message(tr.clone());
                        }
                        continue;
                    }

                    // TODO: user input
                    break;
                }
                FinishReason::Stop => {
                    // TODO: user input
                    break;
                }
            }
        }

        Ok(())
    }

    fn run_tools(&self, tool_calls: &Vec<ToolCall>) -> anyhow::Result<Vec<ToolResult>> {
        let mut results: Vec<ToolResult> = Vec::new();

        for tc in tool_calls {
            match tc.r#type {
                ToolCallType::Function => {
                    let result = self.tool_registry.execute(
                        tc.function.name,
                        &tc.function.arguments,
                        &tc.id,
                        &self.work_dir,
                    );
                    results.push(result);
                }
            }
        }

        Ok(results)
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

fn read_user_input() -> Result<String, anyhow::Error> {
    let mut rl = rustyline::DefaultEditor::new().context("Failed to initialize input editor")?;
    let readline = rl.readline("ragent> ")?;
    Ok(readline.trim().to_string())
}

/// 默认工具注册（生产 + 测试共享）
pub fn default_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(BashTool));
    registry.register(Box::new(ReadFileTool));
    registry.register(Box::new(WriteFileTool));
    registry.register(Box::new(EditFileTool));
    registry.register(Box::new(GlobFileTool));
    registry
}
