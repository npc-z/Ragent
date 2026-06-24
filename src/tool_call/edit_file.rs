use std::fmt::Display;
use std::path::PathBuf;

use serde::Deserialize;

use crate::tool_call::helpers::edit_file;
use crate::tool_call::tool::{FunctionTool, ToolResult};

/// Replace exact text in a file once
#[derive(Debug, Clone, Deserialize)]
pub struct EditFileFunction {
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
    old_text: String,
    new_text: String,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "path={}, old={}, new={}",
            self.path, self.old_text, self.new_text
        )
    }
}

impl EditFileFunction {
    pub fn new(workdir: PathBuf, tool_use_id: String, arguments: String) -> Self {
        let arguments: Arguments = serde_json::from_str(&arguments)
            .expect("failed to parse arguments for EditFileFunction function");

        EditFileFunction {
            workdir,
            tool_use_id,
            arguments,
        }
    }
}

impl FunctionTool for EditFileFunction {
    fn show(&self) {
        println!("EditFileFunction: arguments={}", self.arguments)
    }

    /// Run read file
    fn run(&self) -> ToolResult {
        let path = &self.arguments.path;
        let content = edit_file(
            &self.workdir,
            path,
            self.arguments.old_text.clone(),
            self.arguments.new_text.clone(),
        );

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content,
        }
    }
}
