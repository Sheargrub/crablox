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

    pub fn get(&self, name: &str) -> Result<Literal, String> {
        let iter = self.nodes.iter().rev();
        let mut result: Result<Literal, String> = Err(format!("Undefined variable {}.", name));
        for node in iter {
            if let Some(lit) = node.get(name) {
                result = Ok(lit.clone());
                break;
            }
        }
        result
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
    #[should_panic]
    fn test_env_bad_access() {
        let env = LoxEnvironment::new();
        env.get("fakeVar").expect("Attempted read from undeclared variable");
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
    fn test_env_raise_scope() {
        let mut env = LoxEnvironment::new();
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());

        env.lower_scope();
        env.raise_scope().expect("Scope raise failed");
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }
}