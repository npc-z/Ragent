use std::fmt::Display;
use std::fs::{self};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::tool_call::helpers::safe_path;
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
    pub fn new(workdir: PathBuf, tool_use_id: String, arguments: String) -> Self {
        let arguments: Arguments = serde_json::from_str(&arguments)
            .expect("failed to parse arguments for WriteFileFunction function");

        WriteFileFunction {
            workdir,
            tool_use_id,
            arguments,
        }
    }
}

impl FunctionTool for WriteFileFunction {
    fn show(&self) {
        println!("WriteFileFunction: arguments={}", self.arguments)
    }

    /// Run read file
    fn run(&self) -> ToolResult {
        let path = &self.arguments.path;
        let content = write_file(&self.workdir, path, self.arguments.content.clone());

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content,
        }
    }
}

/// 写入文件
pub fn write_file(workdir: &Path, path: &str, content: String) -> String {
    match safe_path(workdir, path) {
        Ok(path_buf) => match fs::write(&path_buf, content) {
            Ok(_) => format!("Wrote content to {} successful", &path_buf.display()),
            Err(e) => format!("{}", e),
        },
        Err(e) => format!("Error: {}", e),
    }
}
