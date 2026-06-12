use std::fmt::Debug;

pub trait FunctionTool: Debug {
    fn show(&self);
    fn run(&self) -> String;
}
