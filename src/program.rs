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
        let mut parser = LoxParser::new();
        if let Err(v) = parser.load_string(&self.instructions) {
            return format!("Scanning error(s):\n{}", LoxProgram::format_vec_output(v));
        }
        let program = parser.parse();

        match program {
            Ok(p) => {
                let mut interpreter = LoxInterpreter::new();
                match interpreter.interpret(p) {
                    Ok(result) => result,
                    Err(e) => format!("Runtime error: {}", e),
                }
            },
            Err(v) => {
                format!("Parsing error(s):\n{}", LoxProgram::format_vec_output(v))
            },
        }
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