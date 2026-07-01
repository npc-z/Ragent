use std::process::{Command, Output};

use std::{path::Path, process::Stdio};

use serde::Deserialize;
use serde_json::json;

use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::tool::{FunctionTool, ToolResult};

#[derive(Debug)]
pub struct BashTool;

impl FunctionTool for BashTool {
    fn tool_type(&self) -> ToolFunctionType {
        ToolFunctionType::Bash
    }

    fn tool_schema(&self) -> serde_json::Value {
        json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::Bash.as_str(),
                "description": "Run a shell command.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string",},
                    },
                    "required": ["command"],
                },
            },
        })
    }

    fn execute(&self, arguments: &str, tool_use_id: &str, _workdir: &Path) -> ToolResult {
        #[derive(Deserialize)]
        struct Args {
            command: String,
        }

        let args: Args = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => {
                return ToolResult::new(tool_use_id, format!("Error parsing arguments: {}", e));
            }
        };

        if let Some(blocked) = check_dangerous(&args.command) {
            return ToolResult::new(tool_use_id, blocked);
        }

        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                return ToolResult::new(
                    tool_use_id,
                    format!("Error failed to get current directory: {}", e),
                );
            }
        };

        let output = match run_command(&args.command, &cwd) {
            Ok(o) => o,
            Err(e) => {
                return ToolResult::new(
                    tool_use_id,
                    format!("Error: failed to execute command: {}", e),
                );
            }
        };

        ToolResult::new(tool_use_id, format_output(&output))
    }
}

/// 返回 (stdout, stderr) 的合并字符串
fn format_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 同时输出 stdout 和 stderr，带 exit code 前缀
    if output.status.success() {
        if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{}\nstderr:\n{}", stdout, stderr)
        }
    } else {
        format!(
            "exit code: {}\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        )
    }
}

/// 执行 shell 命令，阻塞等待完成
#[cfg(unix)]
fn run_command(command: &str, cwd: &Path) -> std::io::Result<Output> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdin(Stdio::null()) // 防止 shell 等 stdin
        // TODO:
        // .wait_timeout(std::time::Duration::from_secs(30))? // 设置超时
        .output()
}

#[cfg(windows)]
fn run_command(command: &str, cwd: &Path) -> std::io::Result<Output> {
    Command::new("cmd")
        .arg("/C")
        .arg(command)
        .current_dir(cwd)
        .stdin(Stdio::null()) // 防止 shell 等 stdin
        .output()
}

/// 安全检查：返回 Some(error_message) 表示命令应被拦截
fn check_dangerous(command: &str) -> Option<String> {
    // 全量匹配：完全等于这些词就拦截
    let exact_block: &[&str] = &["sudo", "shutdown", "reboot", "halt", "poweroff"];

    for word in exact_block {
        // 命令以该词开头，且后续是空白或结束，确保匹配的是命令名而非参数
        if command == *word
            || command.starts_with(&format!("{} ", word))
            || command.contains(&format!(" {} ", word))
            || command.ends_with(&format!(" {}", word))
            || command.contains(&format!(" {};", word))
        {
            return Some(format!("Error: Dangerous command blocked: {}", command));
        }
    }

    // rm -rf 系列：目标路径是 / 或 /* 开头的拦截
    if command.contains("rm ")
        && (command.contains(" -rf ") || command.contains(" -r "))
        && (command.contains(" /") || command.contains("/*"))
    {
        return Some(format!("Error: Dangerous command blocked: {}", command));
    }

    // /dev/sda 等直接写设备
    if command.contains("dd if=") && command.contains(" of=/dev/")
        || command.contains("mkfs.") && command.contains(" /dev/")
        || command.contains("> /dev/sd")
    {
        return Some(format!("Error: Dangerous command blocked: {}", command));
    }

    None
}
