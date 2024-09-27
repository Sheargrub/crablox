use std::fs;
use std::error::Error;

use crate::lox_interpreter::*;
use crate::lox_parser::*;
use crate::lox_instructions::statement::*;

pub struct LoxProgram {
    instructions : String,
}

impl LoxProgram {
    pub fn build(file_path: &str) -> Result<LoxProgram, Box<dyn Error>> {
        let instructions = fs::read_to_string(file_path)?;
        Ok(LoxProgram{instructions})
    }

    pub fn run(&self) {
        let mut parser = LoxParser::new();
        parser.load_string(&self.instructions).expect("Unhandled");
        let program = parser.parse().expect("Unhandled");

        let mut interpreter = LoxInterpreter::new();
        interpreter.interpret(program);
    }
}