use std::fmt::Display;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::RagentError;
use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::helpers::write_file;
use crate::tool_call::tool::{FunctionTool, ToolResult};

#[derive(Debug, Clone, Deserialize)]
pub struct WriteFileFunction {
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
    content: String,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path={}, content={}", self.path, self.content)
    }
}

impl WriteFileFunction {
    pub fn new(
        workdir: PathBuf,
        tool_use_id: String,
        arguments: String,
    ) -> Result<Self, RagentError> {
        let arguments: Arguments =
            serde_json::from_str(&arguments).map_err(|e| RagentError::InvalidToolArguments {
                tool: ToolFunctionType::WriteFile.as_str().to_string(),
                arguments: arguments.clone(),
                err: e,
            })?;

        Ok(WriteFileFunction {
            workdir,
            tool_use_id,
            arguments,
        })
    }
}

impl FunctionTool for WriteFileFunction {
    fn show(&self) {
        println!("WriteFileFunction: arguments={}", self.arguments)
    }

    /// Run read file
    fn run(&self) -> ToolResult {
        let path = &self.arguments.path;
        let content = match write_file(&self.workdir, path, self.arguments.content.clone()) {
            Ok(s) => s,
            Err(e) => format!("Error writing file {}: {}", path, e),
        };

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content,
        }
    }
}
