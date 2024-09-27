pub mod expression;
mod token;
mod scanner;

use crate::lox_parser::scanner as lox_scanner;
use lox_scanner::*;

use crate::lox_parser::token as lox_token;
use lox_token::*;

use crate::lox_parser::expression as lox_expression;
use lox_expression::Expression;
use lox_expression::node as lox_node;
use lox_node::*;

struct LoxParser {
    tokens: Vec<Token>,
    error_strings: Vec<String>,
    current: usize,
    inited: bool,
    valid: bool,
}

impl LoxParser {

    pub fn new() -> LoxParser {
        let tokens = Vec::new();
        let error_strings = Vec::new();
        let current = 0;
        let inited = false;
        let valid = true;
        LoxParser{tokens, error_strings, current, inited, valid}
    }

    pub fn load_string(&mut self, s: &str) -> Result<(), Vec<String>> {
        let mut scanner = LoxScanner::new(s);
        let scanner_out = scanner.scan_tokens();
        match scanner_out {
            Ok(tokens) => {
                self.load_token_vec(tokens);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn load_token_vec(&mut self, tokens: Vec<Token>) {
        self.tokens = tokens;
        if self.inited && !self.valid { self.error_strings = Vec::new(); }
        self.current = 0;
        self.valid = true;
        self.inited = true;
    }

    /*
    pub fn parse_string(&mut self, tokens: <Vec<Token>>) -> Result<Expression, Vec<String>> {
        let mut current = 0;
    }
    */

    fn expression(&mut self) -> Result<Box<Expression>, ()> {
        let t = self.consume_unsafe();
        let expr = self.equality(t).expect("Unhandled exception.");
        Ok(expr)
    }

    fn equality(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.comparison(t).expect("Unhandled exception.");
        loop {
            match self.peek() {
                Some(Token { data: TokenData::BangEqual, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::NotEqual,
                        self.comparison(right).expect("Unhandled exception."),
                    )
                },
                Some(Token { data: TokenData::EqualEqual, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Equal,
                        self.comparison(right).expect("Unhandled exception."),
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn comparison(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.term(t).expect("Unhandled exception.");
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Less, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Less,
                        self.term(right).expect("Unhandled exception."),
                    )
                },
                Some(Token { data: TokenData::LessEqual, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::LessEqual,
                        self.term(right).expect("Unhandled exception."),
                    )
                },
                Some(Token { data: TokenData::Greater, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Greater,
                        self.term(right).expect("Unhandled exception."),
                    )
                },
                Some(Token { data: TokenData::GreaterEqual, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::GreaterEqual,
                        self.term(right).expect("Unhandled exception."),
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn term(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.factor(t).expect("Unhandled exception.");
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Minus, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Subtract,
                        self.factor(right).expect("Unhandled exception."),
                    )
                },
                Some(Token { data: TokenData::Plus, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Add,
                        self.factor(right).expect("Unhandled exception."),
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn factor(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.unary(t).expect("Unhandled exception.");
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Percent, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Modulo,
                        self.unary(right).expect("Unhandled exception."),
                    );
                },
                Some(Token { data: TokenData::Slash, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Divide,
                        self.unary(right).expect("Unhandled exception."),
                    );
                },
                Some(Token { data: TokenData::Star, line: _ }) => {
                    self.consume().expect("Match statement should prevent None values");
                    let right = self.consume_unsafe();
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Multiply,
                        self.unary(right).expect("Unhandled exception."),
                    );
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn unary(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        match t.data {
            TokenData::Bang => {
                let arg = self.consume_unsafe();
                let u = self.unary(arg).expect("Unhandled exception.");
                Ok(Expression::boxed_not(u))
            },
            TokenData::Minus => {
                let arg = self.consume_unsafe();
                let u = self.unary(arg).expect("Unhandled exception.");
                Ok(Expression::boxed_negative(u))
            },
            _ => Ok(self.primary(t).expect("Unhandled exception.")),
        }
    }

    fn primary(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        match t.data {
            TokenData::Number(n) => Ok(Expression::boxed_number(n)),
            TokenData::StringData(s) => Ok(Expression::boxed_string(&s)),
            TokenData::True => Ok(Expression::boxed_bool(true)),
            TokenData::False => Ok(Expression::boxed_bool(false)),
            TokenData::Nil => Ok(Expression::boxed_nil()),

            TokenData::LeftParen => {
                let e = self.expression().expect("Unhandled exception.");
                if !self.is_at_end() {
                    if let Some(Token{ data: TokenData::RightParen, line: _ }) = self.peek() {
                        self.consume_unsafe();
                    } else {
                        panic!("Unhandled exception: missing close paren.")
                    }
                } else {
                    panic!("Unhandled exception: missing close paren.")
                }
                Ok(Expression::boxed_grouping(e))
            },

            TokenData::EndOfFile => panic!("Unhandled exception: reached end of file unexpectedly."),
            _ => {
                panic!("Unhandled exception: reached impossible state while parsing tokens.");
            },
        }
    }

    fn peek(&self) -> Option<Token> {
        if !self.is_at_end() {
            let t = self.tokens[self.current].clone();
            Some(t)
        }
        else { None }
    }

    // This is a stopgap! Uses of this should be refactored
    fn consume_unsafe(&mut self) -> Token {
        self.consume().expect("Unhandled exception: reached end of token stream early.")
    }

    fn consume(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            let t = self.tokens[self.current].clone();
            self.current += 1;
            Some(t)
        }
        else { None }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_expression_generic(test_str: &str, expected: Box<Expression>) {
        let mut parser = LoxParser::new();
        parser.load_string(test_str).expect("Failed to load test string.");
        let result = parser.expression().expect("Error while parsing expression.");
        // TODO: set up this handler to print out the errors from the parser

        assert_eq!(
            expected,
            result,
            "Expected to recieve left side; recieved right."
        );
    }
    
    mod primary_expressions {
        use super::*;
        
        #[test]
        fn test_expression_primary() {
            let test_str = "\"Hello world!\"";
            let expected = Expression::boxed_string("Hello world!");
            test_expression_generic(test_str, expected);
        }
    }

    mod unary_expressions {
        use super::*;

        #[test]
        fn test_expression_unary_not() {
            let test_str = "!!false";
            let expected = Expression::boxed_not(Expression::boxed_not(Expression::boxed_bool(false)));
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_unary_negative() {
            let test_str = "-4.3";
            let expected = Expression::boxed_negative(Expression::boxed_number(4.3));
            test_expression_generic(test_str, expected);
        }
    }

    mod binary_expressions {
        use super::*;

        #[test]
        fn test_expression_modulo() {
            let test_str = "3 % 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(3.0),
                BinaryOp::Modulo,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_divide() {
            let test_str = "3 / 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(3.0),
                BinaryOp::Divide,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_multiply() {
            let test_str = "4.1 * 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::Multiply,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_add() {
            let test_str = "4.1 + 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::Add,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_subtract() {
            let test_str = "4.1 - 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::Subtract,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_less() {
            let test_str = "4.1 < 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::Less,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_less_equal() {
            let test_str = "4.1 <= 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::LessEqual,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_greater() {
            let test_str = "4.1 > 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::Greater,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_greater_equal() {
            let test_str = "4.1 >= 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::GreaterEqual,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_equal() {
            let test_str = "4.1 == 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::Equal,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_not_equal() {
            let test_str = "4.1 != 5";
            let expected = Expression::boxed_binary(
                Expression::boxed_number(4.1),
                BinaryOp::NotEqual,
                Expression::boxed_number(5.0),
            );
            test_expression_generic(test_str, expected);
        }

    }

    mod compound_expressions {
        use super::*;

        #[test]
        fn test_expression_math_ops() {
            let test_str = "3 + -4 * -5 - 6";
            let expected = Expression::boxed_binary(
                Expression::boxed_binary(
                    Expression::boxed_number(3.0),
                    BinaryOp::Add,
                    Expression::boxed_binary(
                        Expression::boxed_negative(Expression::boxed_number(4.0)),
                        BinaryOp::Multiply,
                        Expression::boxed_negative(Expression::boxed_number(5.0)),
                    ),
                ),
                BinaryOp::Subtract,
                Expression::boxed_number(6.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_comparison() {
            let test_str = "15 % 5 >= 2 != 1.5 + 1.5 < 2";
            let expected = Expression::boxed_binary(
                Expression::boxed_binary(
                    Expression::boxed_binary(
                        Expression::boxed_number(15.0),
                        BinaryOp::Modulo,
                        Expression::boxed_number(5.0),
                    ),
                    BinaryOp::GreaterEqual,
                    Expression::boxed_number(2.0),
                ),
                BinaryOp::NotEqual,
                Expression::boxed_binary(
                    Expression::boxed_binary(
                        Expression::boxed_number(1.5),
                        BinaryOp::Add,
                        Expression::boxed_number(1.5),
                    ),
                    BinaryOp::Less,
                    Expression::boxed_number(2.0),
                ),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_grouping() {
            let test_str = "(3 + -4) * (-5 - 6)";
            let expected = Expression::boxed_binary(
                Expression::boxed_grouping(Expression::boxed_binary(
                    Expression::boxed_number(3.0),
                    BinaryOp::Add,
                    Expression::boxed_negative(Expression::boxed_number(4.0)),
                )),
                BinaryOp::Multiply,
                Expression::boxed_grouping(Expression::boxed_binary(
                    Expression::boxed_negative(Expression::boxed_number(5.0)),
                    BinaryOp::Subtract,
                    Expression::boxed_number(6.0),
                )),
            );
            test_expression_generic(test_str, expected);
        }
    }

}