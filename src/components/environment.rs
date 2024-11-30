use crate::components as lox;
use lox::instructions::node::Literal;
use lox::instructions::callable::Callable;
use std::collections::HashMap;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct LoxEnvironment {
    nodes: Vec<HashMap<String, Literal>>,
    in_closure: bool,
    closure_name: String,
    self_mounts: usize,
    none: Option<Box<LoxEnvironment>>, // hacky mutable constant
}

impl LoxEnvironment  {
    pub fn new() -> LoxEnvironment {
        let nodes = vec![HashMap::new()];
        LoxEnvironment{nodes, in_closure: false, closure_name: String::from(""), self_mounts: 0, none: None}
    }

    fn cur_closure(&mut self) -> &mut Option<Box<LoxEnvironment>> {
        if !self.in_closure {
            return &mut self.none;
        } else {
            let closure_name = self.closure_name.clone();
            match self.get_internal(&closure_name) {
                Ok(Literal::CallLit(Callable::Function(_, _, _, _, closure))) => {
                    closure
                },
                _ => panic!("Mounted nonexistent closure \"{}\".", closure_name),
                // In the second case, something has gone badly wrong
                // and things are likely irrecoverable.
            }
        }
    }

    pub fn define(&mut self, name: &str, mut value: Literal) {
        match &mut self.cur_closure() {
            None => {
                let last = self.nodes.len()-1;
                if let Literal::CallLit(Callable::Function(_, ref mut ref_name, _, _, _)) = value {
                    *ref_name = String::from(name);
                }
                self.nodes[last].insert(String::from(name), value);
            },
            Some(ref mut closure) => {
                closure.define(name, value);
            },
        };
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<Literal, String> {
        match &mut self.cur_closure() {
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
            Some(ref mut closure) => closure.assign(name, value),
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
        match &mut self.cur_closure() {
            None => self.get_internal(name).cloned(),
            Some(ref mut closure) => closure.get(name),
        }
    }

    pub fn lower_scope(&mut self){
        match &mut self.cur_closure() {
            None => {
                self.nodes.push(HashMap::new())
            },
            Some(ref mut closure) => {
                closure.lower_scope()
            },
        }
    }

    pub fn raise_scope(&mut self) -> Result<(), String> {
        match &mut self.cur_closure() {
            None => {
                if self.nodes.len() > 1 {
                    _ = self.nodes.pop();
                    Ok(())
                } else {
                    Err(String::from("Attempted to raise past global scope."))
                }
            },
            Some(ref mut closure) => closure.raise_scope(),
        }
    }

    pub fn spawn_closure(&mut self) -> Box<LoxEnvironment> {
        match &mut self.cur_closure() {
            None => Box::new(self.clone()),
            Some(ref mut closure) => closure.spawn_closure(),
        }
    }

    pub fn mount_closure(&mut self, name: &str) -> Result<(), String> {
        match &mut self.cur_closure() {
            None => {
                if let Ok(Literal::CallLit(Callable::Function(_, _, _, _, closure))) = self.get(name) {
                    self.in_closure = true;
                    self.closure_name = String::from(name);
                    if let None = closure { self.self_mounts += 1; }
                    Ok(())
                }
                else { Err(format!("Attempted to mount nonexistent closure {}.", name)) }
            },
            Some(ref mut closure) => closure.mount_closure(name),
        }
    }

    pub fn unmount_closure(&mut self) -> Result<(), String> {
        match &mut self.cur_closure() {
            None => {
                if self.self_mounts > 0 {
                    self.self_mounts -= 1;
                    self.in_closure = self.self_mounts > 0;
                    Ok(())
                }
                else { Err(format!("Attempted to unmount outermost scope as a closure.")) }
            },
            Some(ref mut closure) => {
                let result = closure.unmount_closure();
                if let Err(_) = result {
                    // Note: if there's any self-mounts below the current mount,
                    // they'll still need to be dealt with.
                    self.in_closure = self.self_mounts > 0;
                }
                Ok(())
            }
        }
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
        let env = LoxEnvironment::new();
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