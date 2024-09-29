use crate::components as lox;
use lox::interpreter::*;
use lox::parser::*;

pub struct LoxPrompt {
    parser : LoxParser,
    interpreter : LoxInterpreter,
}

impl LoxPrompt {
    pub fn new() -> LoxPrompt {
        LoxPrompt {
            parser : LoxParser::new(),
            interpreter : LoxInterpreter::new(),
        }
    }

    pub fn command(&mut self, input: &str) -> String {
        if let Err(v) = self.parser.load_string(input) {
            return format!("Scanning error(s):\n{}", LoxPrompt::format_vec_output(v));
        }
        let program = self.parser.parse();
        
        match program {
            Ok(p) => {
                match self.interpreter.interpret(p) {
                    Ok(result) => result,
                    Err(e) => format!("Runtime error: {}", e),
                }
            },
            Err(v) => {
                format!("Parsing error(s):\n{}", LoxPrompt::format_vec_output(v))
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