use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::expression::*;
use lox::instructions::statement::*;
use lox::interpreter::LoxInterpreter;
use lox::environment::LoxEnvironment;
use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Callable {
    Function(String, String, Vec<String>, Vec<Box<Statement>>, Option<Box<LoxEnvironment>>),
    Class(String, String, Vec<Box<Statement>>),
    Clock,
}

use Callable::*;
impl Callable {
    pub fn native_fn_list() -> Vec<(String, Callable)> {
        vec![(String::from("clock"), Clock)]
    }

    pub fn arity(&self) -> usize {
        match self {
            Function(_, _, arg_names, _, _) => arg_names.len(),
            Class(_, _, _) => 0, // TODO
            Clock => 0,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Function(name, _, _, _, _) => &name,
            Class(name, _, _) => &name,
            Clock => "clock",
        }
    }

    pub fn get_ref_name(&self) -> &str {
        match self {
            Function(_, ref_name, _, _, _) => &ref_name,
            Class(_, ref_name, _) => &ref_name,
            Clock => "clock",
        }
    }

    pub fn set_ref_name(&mut self, new_ref: &str) {
        match self {
            Function(_, ref mut ref_name, _, _, _) => *ref_name = String::from(new_ref),
            Class(_, ref mut ref_name, _) => *ref_name = String::from(new_ref),
            Clock => panic!("Ref name should not be set for native functions"),
        };
    }

    pub fn is_native(&self) -> bool {
        match self {
            Function(_, _, _, _, _) => false,
            Class(_, _, _) => false,
            _ => true,
        }
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Class(name, _, _) => write!(f, "<class {}>", name),
            other => write!(f, "<fn {}>", other.get_name()),
        }
    }
}