use serde::Deserialize;

use crate::llm::deepseek::enums::finish_reason::FinishReason;
use crate::llm::deepseek::enums::model::DeepseekModel;
use crate::llm::deepseek::enums::tool_call_type::ToolCallType;
use crate::llm::response::ApiResponse;
use crate::tool_call::bash::BashFunction;
use crate::tool_call::function_type::ToolFunctionType;
use crate::tool_call::tool::FunctionTool;

#[derive(Clone, Debug, Deserialize)]
pub struct DeepseekResponse {
    id: String,
    created: u64,
    model: DeepseekModel,
    system_fingerprint: String,
    choices: Vec<Choice>,
}

#[derive(Clone, Debug, Deserialize)]
struct Choice {
    index: u16,
    finish_reason: FinishReason,
    message: Message,
}

#[derive(Clone, Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
    reasoning_content: String,
    tool_calls: Vec<ToolCall>,
}

#[derive(Clone, Debug, Deserialize)]
struct ToolCall {
    index: u32,
    id: String,
    r#type: ToolCallType,
    function: ToolCallFunction,
}

#[derive(Clone, Debug, Deserialize)]
struct ToolCallFunction {
    name: ToolFunctionType,
    arguments: String, // e.g. "{\"command\": \"ls -la\"}"
}

impl DeepseekResponse {
    fn get_tool_calls(&self) -> Vec<ToolCall> {
        self.choices.first().unwrap().message.tool_calls.clone()
    }
}

impl ApiResponse for DeepseekResponse {
    fn get_answer(&self) -> String {
        self.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default()
    }

    fn get_reasoning_content(&self) -> String {
        self.choices
            .first()
            .map(|c| c.message.reasoning_content.clone())
            .unwrap_or_default()
    }

    fn get_finishi_reason(&self) -> FinishReason {
        self.choices.first().unwrap().finish_reason
    }

    fn run_tool(&self) -> String {
        // TODO: run all tool
        if let Some(tl) = self.get_tool_calls().into_iter().next() {
            match tl.r#type {
                ToolCallType::Function => match tl.function.name {
                    ToolFunctionType::Bash => {
                        let bf = BashFunction::new(tl.function.arguments);

                        // debug
                        bf.show();

                        let call_result = bf.run();
                        return call_result;
                    }
                },
            }
        }

        "".to_string()
    }

    fn dyr_run_tool(&self) {
        println!(
            "the command to run is: {}",
            self.choices
                .first()
                .unwrap()
                .message
                .tool_calls
                .first()
                .unwrap()
                .function
                .arguments
        );
    }
}
