use serde::Deserialize;
use serde_json::json;

use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::helpers::write_file;
use crate::tool_call::tool::{FunctionTool, ToolResult};

#[derive(Debug)]
pub struct WriteFileTool;

impl FunctionTool for WriteFileTool {
    fn tool_type(&self) -> ToolFunctionType {
        ToolFunctionType::WriteFile
    }

    fn tool_schema(&self) -> serde_json::Value {
        json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::WriteFile.as_str(),
                "description": "Write content to a file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "content": {"type": "string",},
                    },
                    "required": ["path", "content"],
                },
            },
        })
    }

    fn execute(&self, arguments: &str, tool_use_id: &str, workdir: &std::path::Path) -> ToolResult {
        #[derive(Deserialize)]
        struct Args {
            path: String,
            content: String,
        }

        let args: Args = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => {
                return ToolResult::new(tool_use_id, format!("Error parsing arguments: {}", e));
            }
        };

        let output = match write_file(workdir, &args.path, args.content) {
            Ok(s) => s,
            Err(e) => format!("Error writing file {}: {}", args.path, e),
        };

        ToolResult::new(tool_use_id, output)
    }
}
