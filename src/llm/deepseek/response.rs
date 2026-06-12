use serde::Deserialize;

use crate::llm::deepseek::enums::finish_reason::FinishReason;
use crate::llm::deepseek::enums::model::DeepseekModel;
use crate::llm::deepseek::enums::tool_call_type::ToolCallType;
use crate::llm::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct DeepseekResponse {
    id: String,
    created: u64,
    model: DeepseekModel,
    system_fingerprint: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: u16,
    finish_reason: FinishReason,
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
    reasoning_content: String,
    tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Deserialize)]
struct ToolCall {
    index: u32,
    id: String,
    r#type: ToolCallType,
    function: ToolCallFunction,
}

#[derive(Debug, Deserialize)]
struct ToolCallFunction {
    name: String,
    arguments: String, // e.g. "{\"command\": \"ls -la\"}"
}

impl ApiResponse for DeepseekResponse {
    fn get_answer(&self) -> String {
        self.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default()
    }

    fn get_reasoning_content(&self) -> String {
        self.choices
            .first()
            .map(|c| c.message.reasoning_content.clone())
            .unwrap_or_default()
    }
}
