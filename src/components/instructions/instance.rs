use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::expression::*;
use lox::instructions::statement::*;
use lox::instructions::callable::*;
use lox::interpreter::LoxInterpreter;
use lox::environment::LoxEnvironment;
use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Instance {
    class: Callable,
}

impl Instance {
    pub fn new(c: Callable) -> Instance {
        Instance{class: c}
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} instance>", self.class.get_name())
    }
}