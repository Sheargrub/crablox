use std::fs;
use std::error::Error;

use crate::lox_interpreter::*;

pub struct LoxFeeder {
    instructions : String,
}

impl LoxFeeder {
    pub fn build(file_path: &str) -> Result<LoxFeeder, Box<dyn Error>> {
        let instructions = fs::read_to_string(file_path)?;
        Ok(LoxFeeder{instructions})
    }

    // Temporary hedge against interpreter errors.
    pub fn run(self) {
        todo!()
    }
}