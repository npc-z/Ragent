use std::fmt::Display;
use std::process::{Command, Output};

use serde::Deserialize;

use crate::tool_call::tool::{FunctionTool, ToolResult};

#[derive(Debug, Clone, Deserialize)]
pub struct BashFunction {
    /// the tool use id
    pub tool_use_id: String,
    /// function call arguments
    arguments: Arguments,
}

#[derive(Debug, Clone, Deserialize)]
struct Arguments {
    command: String,
}

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "command={}", self.command)
    }
}

impl BashFunction {
    pub fn new(tool_use_id: String, arguments: String) -> Self {
        let arguments: Arguments =
            serde_json::from_str(&arguments).expect("failed to parse arguments for bash function");

        BashFunction {
            tool_use_id,
            arguments,
        }
    }
}

impl FunctionTool for BashFunction {
    fn show(&self) {
        println!("BashCall: arguments={}", self.arguments)
    }

    /// Run bash command
    fn run(&self) -> ToolResult {
        // let dangerous = ["rm -rf /", "sudo", "shutdown", "reboot", "> /dev/"];
        let command = &self.arguments.command;

        if let Some(blocked) = check_dangerous(command) {
            return ToolResult {
                r#type: "tool_result".to_string(),
                tool_use_id: self.tool_use_id.to_string(),
                content: blocked,
            };
        }

        // get current dir
        let cwd = match std::env::current_dir() {
            Ok(d) => d,
            // Err(e) => return format!("Error: failed to get current dir: {}", e),
            Err(e) => panic!("Error: failed to get current dir: {}", e),
        };

        let output = match run_command(command, &cwd) {
            Ok(o) => o,
            // Err(e) => return format!("Error: failed to execute command: {}", e),
            Err(e) => panic!("Error: failed to execute command: {}", e),
        };

        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: self.tool_use_id.to_string(),
            content: format_output(&output),
        }
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
fn run_command(command: &str, cwd: &std::path::Path) -> std::io::Result<Output> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .output()
}

#[cfg(windows)]
fn run_command(command: &str, cwd: &std::path::Path) -> std::io::Result<Output> {
    Command::new("cmd")
        .arg("/C")
        .arg(command)
        .current_dir(cwd)
        .output()
}

/// 安全检查：返回 Some(error_message) 表示命令应被拦截
pub fn check_dangerous(command: &str) -> Option<String> {
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
