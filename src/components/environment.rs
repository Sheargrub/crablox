use crate::components as lox;
use lox::instructions::node::Literal;
use lox::instructions::callable::Callable;

use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct LoxEnvironment {
    nodes: Vec<HashMap<String, Literal>>,
    in_closure: bool,
    cur_closure: Option<RefCell<Box<LoxEnvironment>>>,
    self_mounts: usize,
}

impl LoxEnvironment  {
    pub fn new() -> LoxEnvironment {
        let nodes = vec![HashMap::new()];
        LoxEnvironment{nodes, in_closure: false, cur_closure: None, self_mounts: 0}
    }

    pub fn define(&mut self, name: &str, mut value: Literal) {
        match &mut self.cur_closure {
            None => {
                let last = self.nodes.len()-1;
                match &mut value {
                    Literal::CallLit(Callable::Function(_, ref mut ref_name, _, _, _, _)) => {
                        *ref_name = String::from(name);
                    },
                    Literal::CallLit(Callable::Class(_, ref mut ref_name, _)) => {
                        *ref_name = String::from(name);
                    },
                    _ => (),
                };
                self.nodes[last].insert(String::from(name), value);
            },
            Some(ref mut closure) => {
                closure.borrow_mut().define(name, value);
            },
        };
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<Literal, String> {
        match &mut self.cur_closure {
            None => {
                let iter = self.nodes.iter_mut().rev();
                for node in iter {
                    if node.contains_key(name) {
                        node.insert(String::from(name), value.clone());
                        return Ok(value);
                    }
                }
                Err(format!("Undefined variable {}.", name))
            },
            Some(ref mut closure) => closure.borrow_mut().assign(name, value),
        }
    }

    fn get_internal(&mut self, name: &str) -> Result<&mut Literal, String> {
        let iter = self.nodes.iter_mut().rev();
        for node in iter {
            if let Some(lit) = node.get_mut(name) {
                return Ok(lit);
            }
        }
        Err(format!("Undefined variable {}.", name))
    }

    pub fn get(&mut self, name: &str) -> Result<Literal, String> {
        match &mut self.cur_closure {
            None => self.get_internal(name).cloned(),
            Some(ref mut closure) => closure.borrow_mut().get(name),
        }
    }

    pub fn lower_scope(&mut self){
        match &mut self.cur_closure {
            None => {
                self.nodes.push(HashMap::new())
            },
            Some(ref mut closure) => {
                closure.borrow_mut().lower_scope()
            },
        }
    }

    pub fn raise_scope(&mut self) -> Result<(), String> {
        match &mut self.cur_closure {
            None => {
                if self.nodes.len() > 1 {
                    _ = self.nodes.pop();
                    Ok(())
                } else {
                    Err(String::from("Attempted to raise past global scope."))
                }
            },
            Some(ref mut closure) => closure.borrow_mut().raise_scope(),
        }
    }

    pub fn spawn_closure(&mut self) -> RefCell<Box<LoxEnvironment>> {
        match &mut self.cur_closure {
            None => {
                let mut closure = self.clone();
                closure.decouple_closures();
                RefCell::new(Box::new(closure))
            },
            Some(ref mut closure) => closure.borrow_mut().spawn_closure(),
        }
    }

    fn decouple_closures(&mut self) {
        for node in self.nodes.iter_mut() {
            for (_, val) in node.iter_mut() {
                match val {
                    Literal::CallLit(c) => c.decouple_closures(),
                    Literal::InstLit(i) => i.decouple_closures(),
                    _ => (),
                }
            }
        }
    }

    // Technical note: "None" is used to represent closure self-reference for the sake of preventing loops.
    pub fn mount_closure(&mut self, target_closure: &Option<RefCell<Box<LoxEnvironment>>>) {
        match &mut self.cur_closure {
            None => {
                if let Some(closure) = target_closure {
                    self.in_closure = true;
                    self.cur_closure = Some(closure.clone());
                } else {
                    self.in_closure = true;
                    self.self_mounts += 1;
                }
            },
            Some(ref mut closure) => closure.borrow_mut().mount_closure(target_closure),
        };
    }

    pub fn unmount_closure(&mut self) -> Result<(), String> {
        let mut unset_cur = false;
        match &mut self.cur_closure {
            None => {
                if self.self_mounts > 0 {
                    self.self_mounts -= 1;
                    self.in_closure = self.self_mounts > 0;
                }
                else { return Err(format!("Attempted to unmount outermost scope as a closure.")); }
            },
            Some(ref mut closure) => {
                let result = closure.borrow_mut().unmount_closure();
                if let Err(_) = result {
                    // \/ Check for any underlying self-mounts
                    unset_cur = true;
                    self.in_closure = self.self_mounts > 0;
                }
            }
        };
        if (unset_cur) { self.cur_closure = None; }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_retrieval() {
        let mut env = LoxEnvironment::new();
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }

    #[test]
    fn test_env_assignment() {
        let mut env = LoxEnvironment::new();
        env.define("clicheVar", Literal::Nil);
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.assign("clicheVar", lit_in.clone()).expect("Assignment failed");
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }

    #[test]
    fn test_env_assignment_undeclared() {
        let mut env = LoxEnvironment::new();
        let err_out = env.assign("fake_var", Literal::Boolean(true));
        if let Err(e) = err_out {
            assert!(e.contains("Undefined variable"));
        } else {
            panic!("Unexpectedly recieved valid output.");
        }
    }

    #[test]
    fn test_env_lower_scope() {
        let mut env = LoxEnvironment::new();
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());

        env.lower_scope();
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }

    #[test]
    fn test_env_var_shadowing() {
        let mut env = LoxEnvironment::new();
        env.define("clicheVar", Literal::StringData(String::from("This will get shadowed.")));

        env.lower_scope();
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }

    #[test]
    fn test_env_raise_scope() {
        let mut env = LoxEnvironment::new();
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());

        env.lower_scope();
        env.raise_scope().expect("Scope raise failed");
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }

    #[test]
    fn test_env_access_undeclared() {
        let mut env = LoxEnvironment::new();
        let err_out = env.get("fakeVar");
        if let Err(e) = err_out {
            assert!(e.contains("Undefined variable"));
        } else {
            panic!("Unexpectedly recieved valid output.");
        }
    }

    #[test]
    fn test_env_access_out_of_scope() {
        let mut env = LoxEnvironment::new();
        
        env.lower_scope();
        env.define("clicheVar", Literal::StringData(String::from("Hello world!")));

        env.raise_scope().expect("Scope raise failed");
        let err_out = env.get("clicheVar");
        
        if let Err(e) = err_out {
            assert!(e.contains("Undefined variable"));
        } else {
            panic!("Unexpectedly recieved valid output.");
        }
    }

    #[test]
    fn test_env_raise_past_global() {
        let mut env = LoxEnvironment::new();
        
        env.lower_scope();
        env.raise_scope().expect("Valid scope raise failed");
        let err_out = env.raise_scope();
        
        if let Err(e) = err_out {
            assert!(e.contains("raise past global"));
        } else {
            panic!("Unexpectedly recieved valid output.");
        }
    }
}