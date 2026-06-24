use std::fmt::Display;
use std::fs::{self};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::tool_call::helpers::safe_path;
use crate::tool_call::tool::{FunctionTool, ToolResult};

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
    pub fn new(workdir: PathBuf, tool_use_id: String, arguments: String) -> Self {
        let arguments: Arguments =
            serde_json::from_str(&arguments).expect("failed to parse arguments for bash function");

        ReadFileFunction {
            workdir,
            tool_use_id,
            arguments,
        }
    }
}

impl FunctionTool for ReadFileFunction {
    fn show(&self) {
        println!("BashCall: arguments={}", self.arguments)
    }

    /// Run read file
    fn run(&self) -> ToolResult {
        let path = &self.arguments.path;
        let content = read_file(&self.workdir, path, self.arguments.limit);

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content,
        }
    }
}

/// 读取文件，限制行数（如果 limit 为 Some，则只取前 limit 行并追加提示）。
/// 若发生任何错误，返回以 "Error: " 开头的错误信息。
pub fn read_file(workdir: &Path, path: &str, limit: Option<usize>) -> String {
    match safe_path(workdir, path) {
        Ok(path_buf) => match fs::read_to_string(&path_buf) {
            Ok(content) => {
                let mut lines: Vec<String> = content.lines().map(|x| x.to_string()).collect();
                let total_lines = lines.len();
                if let Some(lim) = limit {
                    if lim < total_lines {
                        lines.truncate(lim);
                        let last_line = format!("... ({} more lines)", total_lines - lim);
                        lines.push(last_line.to_string());
                    }
                }
                lines.join("\n")
            }
            Err(e) => format!("Error: {}", e),
        },
        Err(e) => format!("Error: {}", e),
    }
}
