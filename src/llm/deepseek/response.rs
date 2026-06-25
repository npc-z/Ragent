use serde::Deserialize;

use crate::error::RagentError;
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
    fn get_response_message(&self) -> Result<ResponseMessage, RagentError> {
        let c = self.choices.first().ok_or(RagentError::EmptyResponse)?;
        Ok(c.message.clone())
    }

    fn get_answer(&self) -> String {
        self.get_response_message()
            .map(|m| m.content)
            .unwrap_or_default()
    }

    fn get_reasoning_content(&self) -> String {
        self.get_response_message()
            .map(|m| m.reasoning_content)
            .unwrap_or_default()
    }

    fn get_finishi_reason(&self) -> FinishReason {
        self.choices
            .first()
            .map(|c| c.finish_reason)
            .unwrap_or(FinishReason::Stop) // default to Stop if no choices
    }

    fn get_tool_calls(&self) -> Vec<ToolCall> {
        self.choices
            .first()
            .and_then(|c| c.message.tool_calls.clone())
            .unwrap_or_default()
    }
}
