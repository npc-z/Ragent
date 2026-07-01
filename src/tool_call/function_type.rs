use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolFunctionType {
    Bash,
    ReadFile,
    WriteFile,
    EditFile,
    Glob,
}

impl ToolFunctionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolFunctionType::Bash => "bash",
            ToolFunctionType::ReadFile => "read_file",
            ToolFunctionType::WriteFile => "write_file",
            ToolFunctionType::EditFile => "edit_file",
            ToolFunctionType::Glob => "glob",
        }
    }
}

impl FromStr for ToolFunctionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" => Ok(ToolFunctionType::Bash),
            "read_file" => Ok(ToolFunctionType::ReadFile),
            "write_file" => Ok(ToolFunctionType::WriteFile),
            "edit_file" => Ok(ToolFunctionType::EditFile),
            "glob" => Ok(ToolFunctionType::Glob),
            _ => Err(format!("Unknown tool function type {}", s)),
        }
    }
}
