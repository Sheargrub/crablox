use std::fs;
use std::error::Error;

use crate::lox_interpreter::*;

pub struct LoxFeeder {
    instructions : String,
    interpreter : LoxInterpreter,
}

impl LoxFeeder {
    pub fn build(file_path: &str) -> Result<LoxFeeder, Box<dyn Error>> {
        let instructions = fs::read_to_string(file_path)?;
        let interpreter = LoxInterpreter::new();
        Ok(LoxFeeder{instructions, interpreter})
    }

    pub fn get_source<'a>(&'a self) -> &'a String {
        &self.instructions
    }

    // Temporary hedge against interpreter errors.
    pub fn dummy_call<'a>(&'a self) -> &'a String {
        &self.interpreter.get_instruction()
    }
}