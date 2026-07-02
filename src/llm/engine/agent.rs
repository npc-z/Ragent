use std::path::PathBuf;

use anyhow::Context;
use rustyline::DefaultEditor;

use crate::llm::client::ApiClient;
use crate::llm::deepseek::enums::tool_call_type::ToolCallType;
use crate::llm::deepseek::parser::DeepseekParser;
use crate::llm::engine::conversation::{Conversation, Message};
use crate::llm::response::{FinishReason, ParsedResponse, ResponseMessage};
use crate::tool_call::bash::BashTool;
use crate::tool_call::edit_file::EditFileTool;
use crate::tool_call::glob_file::GlobFileTool;
use crate::tool_call::read_file::ReadFileTool;
use crate::tool_call::tool::{ToolCall, ToolRegistry, ToolResult};
use crate::tool_call::write_file::WriteFileTool;

pub struct Engine {
    client: ApiClient,
    // model: String,
    conversation: Conversation,
    /// tool registry
    tool_registry: ToolRegistry,

    /// 对话轮次
    turn_count: u16,
    work_dir: PathBuf,
    /// user input
    editor: DefaultEditor,
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
        let editor =
            rustyline::DefaultEditor::new().context("Failed to initialize input editor")?;

        Ok(Self::with_deps(client, tool_registry, model, cwd, editor))
    }

    pub fn with_deps(
        client: ApiClient,
        tool_registry: ToolRegistry,
        model: String,
        work_dir: PathBuf,
        editor: DefaultEditor,
    ) -> Self {
        let conversation = Conversation::new(model, tool_registry.schemas());
        let mut e = Self {
            client,
            conversation,
            tool_registry,
            turn_count: 0,
            work_dir,
            editor,
        };
        e.init_messages();
        e
    }

    /// add user message to the body
    fn add_user_message(&mut self, content: String) {
        let msg = Message::User { content };
        print!("{}", &msg);
        self.conversation.push_message(msg);
    }

    /// add system message to the body
    fn add_system_message(&mut self, content: String) {
        let msg = Message::System { content };
        print!("{}", &msg);
        self.conversation.push_message(msg);
    }

    /// add tool call result message to the body
    fn add_tool_call_result_message(&mut self, tool_result: ToolResult) {
        let msg = Message::Tool {
            content: tool_result.content,
            tool_call_id: tool_result.tool_use_id,
        };
        print!("{}", &msg);
        self.conversation.push_message(msg);
    }

    /// add llm response message to the body
    fn add_llm_response_message(&mut self, response: ResponseMessage) {
        let msg = Message::Assistant {
            content: response.content,
            reasoning_content: response.reasoning_content,
            tool_calls: response.tool_calls,
        };
        print!("{}", &msg);
        self.conversation.push_message(msg);
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
        Ok(self.client.send(&self.conversation).await?)
    }

    /// Return None on quit
    fn prompt_user(&mut self) -> anyhow::Result<Option<String>> {
        let user_msg = read_user_input(&mut self.editor)?;
        match user_msg.as_str() {
            "/q" | "/exit" | "/quit" => Ok(None),
            _ => Ok(Some(user_msg)),
        }
    }

    /// Run the message loop, read user input, send message and get the response
    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        // user input
        if let Some(msg) = self.prompt_user()? {
            self.add_user_message(msg);
        } else {
            return Ok(());
        }

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
                        // let the agent goes on with tool results
                        continue;
                    }

                    // user input
                    if let Some(msg) = self.prompt_user()? {
                        self.add_user_message(msg);
                    } else {
                        break;
                    }
                }
                FinishReason::Stop => {
                    // user input
                    if let Some(msg) = self.prompt_user()? {
                        self.add_user_message(msg);
                    } else {
                        break;
                    }
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

fn read_user_input(editor: &mut DefaultEditor) -> Result<String, anyhow::Error> {
    let readline = editor.readline("ragent> ")?;
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
