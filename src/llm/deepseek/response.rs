use serde::Deserialize;

use crate::llm::deepseek::enums::finish_reason::FinishReason;
use crate::llm::response::{ApiResponse, ResponseMessage};
use crate::tool_call::tool::ToolCall;

#[derive(Clone, Debug, Deserialize)]
pub struct DeepseekResponse {
    // id: String,
    // created: u64,
    // model: DeepseekModel,
    // system_fingerprint: String,
    choices: Vec<Choice>,
}

#[derive(Clone, Debug, Deserialize)]
struct Choice {
    // index: u16,
    finish_reason: FinishReason,
    message: ResponseMessage,
}

impl ApiResponse for DeepseekResponse {
    /// get the first llm response message
    fn get_response_message(&self) -> ResponseMessage {
        self.choices
            .first()
            .map(|c| c.message.clone())
            .expect("Can not get response message")
    }

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

    fn get_tool_calls(&self) -> Vec<ToolCall> {
        if let Some(tcs) = self.choices.first().unwrap().message.tool_calls.clone() {
            return tcs;
        };
        Vec::new()
    }
}
