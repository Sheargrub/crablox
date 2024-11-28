use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::expression::*;
use Expression::Identifier;
use lox::instructions::statement::*;
use lox::interpreter::LoxInterpreter;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Callable {
    Function(Box<Expression>), // TODO
    Clock,
}

use Callable::*;
impl Callable {
    pub fn native_fn_list() -> Vec<(String, Callable)> {
        vec![(String::from("clock"), Clock)]
    }

    pub fn arity(&self) -> usize {
        match self {
            Function(e) => {
                todo!();
            },
            Clock => 0,
        }
    }

    pub fn call(&self, args: Vec<Box<Expression>>, interpreter: &LoxInterpreter) -> Result<Literal, String> {
        match self {
            Function(e) => {
                todo!();
            },
            Clock => {
                let now = SystemTime::now();
                let time_ms = now.duration_since(UNIX_EPOCH).expect("Got time before unix epoch").as_millis() as f64;
                Ok(Literal::Number(time_ms/1000.0))
            },
        }
    }
}