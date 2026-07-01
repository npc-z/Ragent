use crate::{
    error::RagentError,
    llm::{
        deepseek::response::DeepseekResponse,
        response::{ParsedResponse, ResponseParser},
    },
};

#[derive(Debug)]
pub struct DeepseekParser;

impl ResponseParser for DeepseekParser {
    fn parse(&self, text: &str, llm_name: &str) -> Result<ParsedResponse, RagentError> {
        let raw: DeepseekResponse =
            serde_json::from_str(text).map_err(|e| RagentError::ApiParse {
                llm: llm_name.to_string(),
                e: e.to_string(),
            })?;

        raw.into_parsed()
    }
}
