pub fn new_error_string(line: usize, message: &str) -> String {
    format!("[Line {}] Error: {}", line, message)
}


#[cfg(test)]
mod error_tests {
    use crate::lox_envs::components as lox;

    #[test]
    fn generate_err_test () {
        let error_string = lox::error::new_error_string(5, "Insufficient crabs");
        assert_eq!(
            String::from("[Line 5] Error: Insufficient crabs"),
            error_string
        );
    }

}