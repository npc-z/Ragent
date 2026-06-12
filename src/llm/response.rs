use std::fmt::Debug;

use crate::llm::deepseek::enums::finish_reason::FinishReason;

pub trait ApiResponse: Clone + Debug {
    // get the llm answer
    fn get_answer(&self) -> String;

    // get the llm reasoning content
    fn get_reasoning_content(&self) -> String;

    // get teh finish reason
    fn get_finishi_reason(&self) -> FinishReason;

    fn dyr_run_tool(&self);

    fn run_tool(&self) -> String;
}
