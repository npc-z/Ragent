use std::{fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{error::RagentError, tool_call::tool::ToolCall};

/// LLM 协议层的 finish reason（非 provider 特定）
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    ToolCalls,
    Stop,
}

impl FinishReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            FinishReason::ToolCalls => "tool_calls",
            FinishReason::Stop => "stop",
        }
    }
}

impl FromStr for FinishReason {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tool_calls" => Ok(FinishReason::ToolCalls),
            "stop" => Ok(FinishReason::Stop),
            _ => Err(format!("Unknown finish_reason {}", s)),
        }
    }
}

/// Provider 无关的 LLM 响应
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 解析后的完整响应
#[derive(Clone, Debug)]
pub struct ParsedResponse {
    pub message: ResponseMessage,
    pub finish_reason: FinishReason,
    pub tool_calls: Vec<ToolCall>,
}

/// Provider 需要实现解析
pub trait ResponseParser: Send + Sync + Debug {
    fn parse(&self, text: &str, llm_name: &str) -> Result<ParsedResponse, RagentError>;
}
