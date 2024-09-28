use crate::components as lox;
use lox::instructions::node::Literal;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LoxEnvironment {
    nodes: Vec<HashMap<String, Literal>>,
}

impl LoxEnvironment  {
    pub fn new() -> LoxEnvironment {
        let nodes = vec![HashMap::new()];
        LoxEnvironment{nodes}
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        let last = self.nodes.len()-1;
        self.nodes[last].insert(String::from(name), value);
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<Literal, String> {
        let iter = self.nodes.iter_mut().rev();
        for node in iter {
            if node.contains_key(name) {
                node.insert(String::from(name), value.clone());
                return Ok(value);
            }
        }
        Err(format!("Undefined variable {}.", name))
    }

    pub fn get(&self, name: &str) -> Result<Literal, String> {
        let iter = self.nodes.iter().rev();
        for node in iter {
            if let Some(lit) = node.get(name) {
                return Ok(lit.clone());
            }
        }
        Err(format!("Undefined variable {}.", name))
    }

    pub fn lower_scope(&mut self){
        self.nodes.push(HashMap::new());
    }

    pub fn raise_scope(&mut self) -> Result<(), String> {
        if self.nodes.len() > 1 {
            _ = self.nodes.pop();
            Ok(())
        } else {
            Err(String::from("Attempted to raise past global scope."))
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
        env.assign("fake_var", Literal::Boolean(true));
        let err_out = env.get("fakeVar");
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