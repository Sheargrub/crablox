pub mod lox_envs {
    pub mod program;
    mod components;
}

use crate::lox_envs::program::*;

pub fn run_file(file_name: &str) {
    println!("Planning to run file {}.", file_name);
    let file_runner = LoxProgram::build(file_name).unwrap_or_else(|_err| {
        panic!("Unhandled error opening file.")
    });
    file_runner.run();
}

pub fn run_prompt() {
    println!("Planning to initialize command prompt.");
}