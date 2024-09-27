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

    // Temporary hedge against interpreter errors.
    pub fn dummy_call(self) {
        println!("Hello world!")
    }
}