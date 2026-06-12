use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LlmType {
    DeepSeek,
}

impl LlmType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmType::DeepSeek => "DeepSeek",
        }
    }
}

impl Display for LlmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LlmType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DeepSeek" => Ok(LlmType::DeepSeek),
            _ => Err(format!("Unknown llm type {}", s)),
        }
    }
}
