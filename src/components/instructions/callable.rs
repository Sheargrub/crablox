use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::expression::*;
use Expression::Identifier;
use lox::instructions::statement::*;
use lox::interpreter::LoxInterpreter;

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
                let now = std::time::Instant::now();
                Ok(Literal::Number(now.elapsed().as_secs_f64()))
            },
        }
    }
}