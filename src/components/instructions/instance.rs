use crate::components as lox;
use lox::instructions::node::*;
use lox::instructions::callable::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Instance {
    class: Callable,
    fields: Rc<RefCell<HashMap<String, Literal>>>,
}

impl Instance {
    pub fn new(class: Callable) -> Instance {
        Instance{class, fields: Rc::new(RefCell::new(HashMap::new()))}
    }

    pub fn get(&self, name: &str) -> Result<Literal, String> {
        if let Some(lit) = self.fields.borrow().get(name) {
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
        self.fields.borrow_mut().insert(String::from(name), value);
    }

    pub fn decouple_closures(&mut self) {
        self.class.decouple_closures();
        let fields = self.fields.borrow().clone();
        self.fields = Rc::new(RefCell::new(fields));
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} instance>", self.class.get_name())
    }
}