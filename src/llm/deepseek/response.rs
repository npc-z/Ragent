use serde::Deserialize;

use crate::error::RagentError;
use crate::llm::response::{FinishReason, ParsedResponse, ResponseMessage};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DeepseekResponse {
    // id: String,
    // created: u64,
    // model: DeepseekModel,
    // system_fingerprint: String,
    choices: Vec<Choice>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Choice {
    // index: u16,
    finish_reason: FinishReason,
    message: ResponseMessage,
}

impl DeepseekResponse {
    pub(crate) fn into_parsed(self) -> Result<ParsedResponse, RagentError> {
        let c = self
            .choices
            .into_iter()
            .next()
            .ok_or(RagentError::EmptyResponse)?;
        Ok(ParsedResponse {
            finish_reason: c.finish_reason,
            tool_calls: c.message.tool_calls.clone().unwrap_or_default(),
            message: c.message,
        })
    }
}
