use serde::Deserialize;
use serde_json::json;

use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::helpers::edit_file;
use crate::tool_call::tool::{FunctionTool, ToolResult};

/// Replace exact text in a file once
#[derive(Debug)]
pub struct EditFileTool;

impl FunctionTool for EditFileTool {
    fn tool_type(&self) -> ToolFunctionType {
        ToolFunctionType::EditFile
    }

    fn tool_schema(&self) -> serde_json::Value {
        json!({
            "type": "function",
            "function": {
                "name": ToolFunctionType::EditFile.as_str(),
                "description": "Replace exact text in a file once",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string",},
                        "old_text": {"type": "string",},
                        "new_text": {"type": "string",},
                    },
                    "required": ["path", "old_text", "new_text"],
                },
            },
        })
    }

    fn execute(&self, arguments: &str, tool_use_id: &str, workdir: &std::path::Path) -> ToolResult {
        #[derive(Deserialize)]
        struct Args {
            path: String,
            old_text: String,
            new_text: String,
        }

        let args: Args = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => {
                return ToolResult::new(tool_use_id, format!("Error parsing arguments: {}", e));
            }
        };

        let output = match edit_file(workdir, &args.path, args.old_text, args.new_text) {
            Ok(s) => s,
            Err(e) => format!("Error editing file {}: {}", args.path, e),
        };

        ToolResult::new(tool_use_id, output)
    }
}
