use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolFunctionType {
    Bash,
}

impl ToolFunctionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolFunctionType::Bash => "bash",
        }
    }
}

impl FromStr for ToolFunctionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" => Ok(ToolFunctionType::Bash),
            _ => Err(format!("Unknown tool function type {}", s)),
        }
    }
}
