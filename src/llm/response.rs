use std::fmt::Debug;

pub trait ApiResponse: Debug {
    // get the llm answer
    fn get_answer(&self) -> String;

    // get the llm reasoning content
    fn get_reasoning_content(&self) -> String;
}
