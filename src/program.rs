use crate::components as lox;
use lox::interpreter::*;
use lox::parser::*;
use std::fs;
use std::error::Error;

pub struct LoxProgram {
    instructions : String,
}

impl LoxProgram {
    pub fn build(file_path: &str) -> Result<LoxProgram, Box<dyn Error>> {
        let instructions = fs::read_to_string(file_path)?;
        Ok(LoxProgram{instructions})
    }

    pub fn run(&self) -> String {
        let mut output = String::new();

        let mut parser = LoxParser::new();
        parser.load_string(&self.instructions).expect("TODO: Unhandled failure when loading to parser");
        let program = parser.parse();

        match program {
            Ok(p) => {
                let mut interpreter = LoxInterpreter::new();
                match interpreter.interpret(p) {
                    Ok(result) => output = result,
                    Err(e) => {
                        output.push_str("Runtime error:\n");
                        output.push_str(&e);
                    }
                };
            },
            Err(v) => {
                output.push_str("Parsing error(s):\n");
                output.push_str(&LoxProgram::format_vec_output(v));
            },
        }

        output
    }

    fn format_vec_output(v: Vec<String>) -> String {
        let mut output = String::new();
        for s in v {
            output.push_str(&s);
            output.push_str("\n");
        }
        output.pop();
        output
    }
}