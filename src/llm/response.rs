use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{llm::deepseek::enums::finish_reason::FinishReason, tool_call::tool::ToolCall};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub trait ApiResponse: Clone + Debug {
    /// get the first llm response message
    fn get_response_message(&self) -> ResponseMessage;

    /// get the llm answer
    fn get_answer(&self) -> String;

    /// get the llm reasoning content
    fn get_reasoning_content(&self) -> String;

    /// get the finish reason
    fn get_finishi_reason(&self) -> FinishReason;

    /// get all tool calls query
    fn get_tool_calls(&self) -> Vec<ToolCall>;
}
