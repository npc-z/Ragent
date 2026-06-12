use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
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
