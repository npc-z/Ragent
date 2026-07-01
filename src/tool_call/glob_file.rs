use serde::Deserialize;
use serde_json::json;

use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::helpers::glob_file;
use crate::tool_call::tool::{FunctionTool, ToolResult};

/// Find files matching a glob pattern
#[derive(Debug)]
pub struct GlobFileTool;

impl FunctionTool for GlobFileTool {
    fn tool_type(&self) -> ToolFunctionType {
        ToolFunctionType::Glob
    }

    fn tool_schema(&self) -> serde_json::Value {
        json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::Glob.as_str(),
                "description": "Find files matching a glob pattern",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "pattern": {"type": "string",},
                    },
                    "required": ["path","pattern"],
                },
            },
        })
    }

    fn execute(&self, arguments: &str, tool_use_id: &str, workdir: &std::path::Path) -> ToolResult {
        #[derive(Deserialize)]
        struct Args {
            path: String,
            pattern: String,
        }

        let args: Args = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => {
                return ToolResult::new(tool_use_id, format!("Error parsing arguments: {}", e));
            }
        };

        let output = match glob_file(workdir, &args.path, args.pattern) {
            Ok(s) => s,
            Err(e) => format!("Error globing file {}: {}", args.path, e),
        };

        ToolResult::new(tool_use_id, output)
    }
}
