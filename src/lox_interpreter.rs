use crate::lox_scanner::*;

pub struct LoxInterpreter {
    instruction : String,
}

impl LoxInterpreter {
    pub fn new() -> LoxInterpreter {
        let instruction = String::from("unknown");
        LoxInterpreter{instruction}
    }

    pub fn get_instruction<'a>(&'a self) -> &'a String {
        &self.instruction
    }
}
