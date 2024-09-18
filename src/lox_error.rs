
pub struct LoxError {
    line : u32,
    message : String,
}

impl LoxError {
    pub fn new(line : u32, message : &str) -> LoxError {
        LoxError{line, message: String::from(message)}
    }
    
    pub fn to_string(&self) -> String {
        // todo: unclear what "where" is for in the original function?
        format!("[Line {}] Error: {}", self.line, self.message)
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn error_to_string () {
        let my_error = LoxError::new(5, "Insufficient crabs");
        assert_eq!(
            "[Line 5] Error: Insufficient crabs", 
            my_error.to_string(),
        );
    }

}