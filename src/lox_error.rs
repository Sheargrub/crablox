

pub fn generate_err<T>(line: usize, message: &str) -> Result<T, String> {
    Err(format!("[Line {}] Error: {}", line, message))
}


#[cfg(test)]
mod error_tests {
    use crate::lox_error;

    #[test]
    fn generate_err_test () {
        let my_error = lox_error::generate_err::<usize>(5, "Insufficient crabs");
        assert_eq!(
            Err(String::from("[Line 5] Error: Insufficient crabs")),
            my_error
        );
    }

}