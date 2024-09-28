use crate::components as lox;
use lox::instructions::node::Literal;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LoxEnvironment<'a> {
    values: HashMap<String, Literal>,
    parent: Option<&'a Box<LoxEnvironment<'a>>>,
}

impl<'a> LoxEnvironment<'a>  {
    pub fn new() -> LoxEnvironment<'a> {
        let values = HashMap::new();
        LoxEnvironment{values, parent: None}
    }

    pub fn new_below(parent: &'a Box<LoxEnvironment<'a>>) -> LoxEnvironment<'a> {
        let values = HashMap::new();
        LoxEnvironment{values, parent: Some(parent)}
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.values.insert(String::from(name), value);
    }

    pub fn get(&self, name: &str) -> Result<Literal, String> {
        if self.values.contains_key(name) {
            let out = self.values.get(name).expect(".contains_key() check should guarantee .get()");
            Ok(out.clone())
        } else {
            if let Some(b) = &self.parent {
                b.get(name)
            } else {
                Err(format!("Undefined variable {}.", name))
            }
        }
    }

    pub fn raise_scope(&self) -> Result<&Box<LoxEnvironment>, String> {
        if let Some(b) = self.parent { Ok(b) }
        else { Err(String::from("Attempted to elevate beyond root scope.")) }
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
        let mut env = &mut Box::new(LoxEnvironment::new());
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());

        let mut env = &mut Box::new(LoxEnvironment::new_below(env));
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }

    #[test]
    fn test_env_raise_scope() {
        let mut env = &mut Box::new(LoxEnvironment::new());
        let lit_in = Literal::StringData(String::from("Hello world!"));
        env.define("clicheVar", lit_in.clone());
        
        let mut env = &mut Box::new(LoxEnvironment::new_below(env));

        let mut env = &mut env.raise_scope().expect("Raising scope failed");
        let lit_out = env.get("clicheVar").expect("Read failed");
        assert_eq!(lit_in, lit_out, "Environment output a different value than was put in.");
    }
}