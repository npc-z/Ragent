use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    llm::deepseek::enums::tool_call_type::ToolCallType, tool_call::function_type::ToolFunctionType,
};

pub trait FunctionTool: Debug {
    fn show(&self);
    fn run(&self) -> ToolResult;
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    /// only tool_result now
    pub r#type: String,
    /// the tool use id
    pub tool_use_id: String,
    /// the tool call result
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCall {
    /// tool index
    pub index: u32,
    /// tool call id
    pub id: String,
    pub r#type: ToolCallType,
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCallFunction {
    pub name: ToolFunctionType,
    pub arguments: String, // e.g. "{\"command\": \"ls -la\"}"
}
