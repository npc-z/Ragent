use std::{collections::HashMap, fmt::Debug, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    llm::deepseek::enums::tool_call_type::ToolCallType, tool_call::function_type::ToolFunctionType,
};

pub trait FunctionTool: Debug {
    /// 返回工具函数类型
    fn tool_type(&self) -> ToolFunctionType;

    /// 工具定义
    fn tool_schema(&self) -> Value;

    /// 执行工具函数
    fn execute(&self, arguments: &str, tool_use_id: &str, workdir: &Path) -> ToolResult;
}

pub struct ToolRegistry {
    tools: HashMap<ToolFunctionType, Box<dyn FunctionTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register(&mut self, tool: Box<dyn FunctionTool>) {
        self.tools.insert(tool.tool_type(), tool);
    }

    /// 所有工具的定义
    pub fn schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| t.tool_schema()).collect()
    }

    /// 调用工具
    pub fn execute(
        &self,
        name: ToolFunctionType,
        arguments: &str,
        tool_use_id: &str,
        workdir: &Path,
    ) -> ToolResult {
        match self.tools.get(&name) {
            Some(tool) => tool.execute(arguments, tool_use_id, workdir),
            None => ToolResult::new(tool_use_id, format!("Tool not found: {:?}", name)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    /// only tool_result now
    pub r#type: String,
    /// the tool use id
    pub tool_use_id: String,
    /// the tool call result
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCall {
    /// tool index
    pub index: u32,
    /// tool call id
    pub id: String,
    pub r#type: ToolCallType,
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCallFunction {
    pub name: ToolFunctionType,
    pub arguments: String, // e.g. "{\"command\": \"ls -la\"}"
}

impl ToolResult {
    pub fn new(tool_use_id: &str, content: String) -> Self {
        ToolResult {
            r#type: "tool_result".to_string(),
            tool_use_id: tool_use_id.to_string(),
            content,
        }
    }
}
