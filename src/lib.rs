pub mod program;
pub mod prompt;
mod components{
    pub mod parser;
    pub mod interpreter;
    pub mod instructions;

    mod error;
    mod environment;
}

use crate::program::*;
use crate::prompt::*;
use std::io;
use std::io::Write;

pub fn run_file(file_name: &str) {
    println!("Planning to run file {}.", file_name);
    let file_runner = LoxProgram::build(file_name).unwrap_or_else(|_err| {
        panic!("Unhandled error opening file.")
    });
    println!("{}", file_runner.run());
}

pub fn run_prompt() {
    println!("Planning to initialize command prompt.");
    let mut prompt = LoxPrompt::new();
    
    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().expect("Fatal IO error");
        io::stdin().read_line(&mut input).expect("Failed to read line");
        println!("{}", prompt.command(&input));
    }
}