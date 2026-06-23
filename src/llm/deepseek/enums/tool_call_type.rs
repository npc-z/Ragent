use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallType {
    Function,
}

impl ToolCallType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolCallType::Function => "function",
        }
    }
}

impl FromStr for ToolCallType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "function" => Ok(ToolCallType::Function),
            _ => Err(format!("Unknown tool call type {}", s)),
        }
    }
}
