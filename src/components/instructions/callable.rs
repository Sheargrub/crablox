use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::expression::*;
use Expression::Identifier;
use lox::instructions::statement::*;
use lox::interpreter::LoxInterpreter;
use lox::environment::LoxEnvironment;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Callable {
    Function(String, Vec<String>, Vec<Box<Statement>>),
    Clock,
}

use Callable::*;
impl Callable {
    pub fn native_fn_list() -> Vec<(String, Callable)> {
        vec![(String::from("clock"), Clock)]
    }

    pub fn arity(&self) -> usize {
        match self {
            Function(name, arg_names, body) => arg_names.len(),
            Clock => 0,
        }
    }

    pub fn call(&self, args: Vec<Literal>, interpreter: &mut LoxInterpreter) -> Result<Literal, String> {
        match self {
            Function(name, arg_names, body) => {
                // PRECONDITION: Scope should be raised before calling this!
                let mut name_iter = arg_names.iter().peekable();
                let mut arg_iter = args.iter().peekable();
                while name_iter.peek() != None && arg_iter.peek() != None {
                    interpreter.define_external(
                        name_iter.next().unwrap_or_else(|| panic!("Impossible unwrap fail")),
                        arg_iter.next().unwrap_or_else(|| panic!("Impossible unwrap fail")).clone(),
                    );
                }
                let result = interpreter.evaluate_stmt(Statement::Block(body.clone()));
                match result {
                    Ok(_) => Ok(Literal::Nil),
                    Err(s) => Err(s),
                }
            },
            Clock => {
                let now = SystemTime::now();
                let time_ms = now.duration_since(UNIX_EPOCH).expect("Got time before unix epoch").as_millis() as f64;
                Ok(Literal::Number(time_ms/1000.0))
            },
        }
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callable::Function(name, _, _) => write!(f, "<fn {}>", name),
            Callable::Clock => write!(f, "<fn clock>")
        }
    }
}