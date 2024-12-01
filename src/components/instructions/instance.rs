use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::expression::*;
use lox::instructions::statement::*;
use lox::instructions::callable::*;
use lox::interpreter::LoxInterpreter;
use lox::environment::LoxEnvironment;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Instance {
    class: Callable,
    fields: HashMap<String, Literal>,
}

impl Instance {
    pub fn new(class: Callable) -> Instance {
        Instance{class, fields: HashMap::new()}
    }

    pub fn get(&self, name: &str) -> Result<Literal, String> {
        if let Some(lit) = self.fields.get(name) {
            Ok(lit.clone())
        } else {
            if let Ok(c) = self.class.find_method(name) {
                Ok(Literal::CallLit(c))
            } else {
                Err(format!("Undefined property {}.", name))
            }
        }
    }

    pub fn set(&mut self, name: &str, value: Literal) {
        self.fields.insert(String::from(name), value);
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} instance>", self.class.get_name())
    }
}