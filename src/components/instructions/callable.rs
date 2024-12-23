use crate::components as lox;
use lox::instructions::statement::*;
use lox::environment::LoxEnvironment;

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Callable {
    Function(String, Vec<String>, Vec<Box<Statement>>, Option<Rc<RefCell<LoxEnvironment>>>, bool),
    Class(String, Option<Box<Callable>>, HashMap<String, Callable>),
    Clock,
}

use Callable::*;
impl Callable {
    pub fn native_fn_list() -> Vec<(String, Callable)> {
        vec![(String::from("clock"), Clock)]
    }

    pub fn arity(&self) -> usize {
        match self {
            Function(_, arg_names, _, _, _) => arg_names.len(),
            Class(_, _, methods) => {
                if let Some(c) = methods.get("init") { c.arity() }
                else { 0 }
            },
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

    pub fn is_native(&self) -> bool {
        match self {
            Function(_, _, _, _, _) => false,
            Class(_, _, _) => false,
            _ => true,
        }
    }

    pub fn is_initializer(&self) -> bool {
        match self {
            Function(_, _, _, _, is_init) => *is_init,
            Class(_, _, _) => false,
            _ => true,
        }
    }

    pub fn find_method(&self, name: &str) -> Result<Callable, String> {
        match self {
            Class(_, super_class, methods) => {
                if let Some(c) = methods.get(name) {
                    Ok(c.clone())
                } else if let Some(sc) = super_class {
                    sc.find_method(name)
                } else {
                    Err(format!("Undefined property {}.", name))
                }
            }
            _ => Err(format!("Cannot find method on non-class {}.", self.get_name())),
        }
    }

    pub fn decouple_closures(&mut self) {
        match self {
            Function(_, _, _, ref mut closure, _) => {
                let mut temp = None;
                if let Some(inner) = closure {
                    temp = Some(inner.borrow_mut().spawn_closure());
                }
                *closure = temp;
            },
            Class(_, _, methods) => {
                for (_, m) in methods {
                    m.decouple_closures();
                }
            },
            _ => (),
        };
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