use std::fmt::Display;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::RagentError;
use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::helpers::glob_file;
use crate::tool_call::tool::{FunctionTool, ToolResult};

/// Replace exact text in a file once
#[derive(Debug, Clone, Deserialize)]
pub struct GlobFileFunction {
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
    pattern: String,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path={}, pattern={}", self.path, self.pattern)
    }
}

impl GlobFileFunction {
    pub fn new(
        workdir: PathBuf,
        tool_use_id: String,
        arguments: String,
    ) -> Result<Self, RagentError> {
        let arguments: Arguments =
            serde_json::from_str(&arguments).map_err(|e| RagentError::InvalidToolArguments {
                tool: ToolFunctionType::Glob.as_str().to_string(),
                arguments: arguments.clone(),
                err: e,
            })?;

        Ok(GlobFileFunction {
            workdir,
            tool_use_id,
            arguments,
        })
    }
}

impl FunctionTool for GlobFileFunction {
    fn show(&self) {
        println!("GlobFileFunction: arguments={}", self.arguments)
    }

    /// Run read file
    fn run(&self) -> ToolResult {
        let path = &self.arguments.path;
        let content = match glob_file(&self.workdir, path, self.arguments.pattern.clone()) {
            Ok(s) => s,
            Err(e) => format!("Error glob file {}: {}", path, e),
        };

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content,
        }
    }
}
