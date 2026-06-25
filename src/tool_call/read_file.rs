use std::fmt::Display;
use std::path::PathBuf;

use serde::Deserialize;

use crate::{
    error::RagentError,
    tool_call::{
        function_type::ToolFunctionType,
        helpers::read_file,
        tool::{FunctionTool, ToolResult},
    },
};

#[derive(Debug, Clone, Deserialize)]
pub struct ReadFileFunction {
    /// work dir
    workdir: PathBuf,
    /// the tool use id
    pub tool_use_id: String,
    /// function call arguments
    arguments: Arguments,
}

#[derive(Debug, Clone, Deserialize)]
struct Arguments {
    path: String,
    limit: Option<usize>,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path={}, limit={:?}", self.path, self.limit)
    }
}

impl ReadFileFunction {
    pub fn new(
        workdir: PathBuf,
        tool_use_id: String,
        arguments: String,
    ) -> Result<Self, RagentError> {
        let arguments: Arguments =
            serde_json::from_str(&arguments).map_err(|e| RagentError::InvalidToolArguments {
                tool: ToolFunctionType::ReadFile.as_str().to_string(),
                arguments: arguments.clone(),
                err: e,
            })?;

        Ok(ReadFileFunction {
            workdir,
            tool_use_id,
            arguments,
        })
    }
}

impl FunctionTool for ReadFileFunction {
    fn show(&self) {
        println!("ReadFileFunction: arguments={}", self.arguments)
    }

    /// Run read file
    fn run(&self) -> ToolResult {
        let path = &self.arguments.path;
        let content = match read_file(&self.workdir, path, self.arguments.limit) {
            Ok(s) => s,
            Err(e) => format!("Error reading file {}: {}", path, e),
        };

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content,
        }
    }
}
