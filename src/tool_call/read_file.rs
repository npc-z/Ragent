use serde::Deserialize;
use serde_json::json;

use crate::tool_call::{
    function_type::ToolFunctionType,
    helpers::read_file,
    tool::{FunctionTool, ToolResult},
};

#[derive(Debug)]
pub struct ReadFileTool;

impl FunctionTool for ReadFileTool {
    fn tool_type(&self) -> ToolFunctionType {
        ToolFunctionType::ReadFile
    }

    fn tool_schema(&self) -> serde_json::Value {
        json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::ReadFile.as_str(),
                "description": "Read file contents.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "limit": {"type": "integer",},
                    },
                    "required": ["path"],
                },
            },
        })
    }

    fn execute(&self, arguments: &str, tool_use_id: &str, workdir: &std::path::Path) -> ToolResult {
        #[derive(Deserialize)]
        struct Args {
            path: String,
            limit: Option<usize>,
        }

        let args: Args = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => {
                return ToolResult::new(tool_use_id, format!("Error parsing arguments: {}", e));
            }
        };

        let output = match read_file(workdir, &args.path, args.limit) {
            Ok(s) => s,
            Err(e) => format!("Error reading file {}: {}", args.path, e),
        };

        ToolResult::new(tool_use_id, output)
    }
}
