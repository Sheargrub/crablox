
pub mod lox_instructions;
pub mod lox_parser;
pub mod lox_interpreter;
pub mod lox_feeder;
pub mod lox_error;

use crate::lox_interpreter::*;
use crate::lox_feeder::*;

pub fn run_file(file_name: &str) {
    println!("Planning to run file {}.", file_name);
    let file_runner = LoxFeeder::build(file_name).unwrap_or_else(|_err| {
        panic!("Unhandled error opening file.")
    });
    println!("{}", file_runner.get_source());
    println!("\n{}", file_runner.dummy_call()); // temp
}

pub fn run_prompt() {
    println!("Planning to initialize command prompt.");
    let interpreter = LoxInterpreter::new();
    println!("{}", interpreter.get_instruction());
}