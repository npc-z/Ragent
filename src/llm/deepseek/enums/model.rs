use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeepseekModel {
    DeepseekV4Pro,
    DeepseekV4Flash,
}

impl DeepseekModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeepseekModel::DeepseekV4Pro => "deepseek-v4-pro",
            DeepseekModel::DeepseekV4Flash => "deepseek-v4-flash",
        }
    }
}

impl FromStr for DeepseekModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deepseek-v4-pro" => Ok(DeepseekModel::DeepseekV4Pro),
            "deepseek-v4-flash" => Ok(DeepseekModel::DeepseekV4Flash),
            _ => Err(format!("Unknown model name {}", s)),
        }
    }
}
