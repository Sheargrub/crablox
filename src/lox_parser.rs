mod token;
mod scanner;

use crate::lox_parser::scanner as lox_scanner;
use lox_scanner::*;

use crate::lox_parser::token as lox_token;
use lox_token::*;

use crate::lox_instructions::{expression as lox_expression, node as lox_node};
use lox_expression::Expression;
use lox_node::*;

use crate::lox_error;

pub struct LoxParser {
    tokens: Vec<Token>,
    error_strings: Vec<String>,
    output: Expression,
    current: usize,
    line: usize,
    inited: bool,
    loaded: bool,
    valid: bool,
}

impl LoxParser {

    pub fn new() -> LoxParser {
        let tokens = Vec::new();
        let error_strings = Vec::new();
        let output = Expression::LExp(Literal::Nil);
        let current = 0;
        let line = 1;
        let inited = false;
        let loaded = false;
        let valid = true;
        LoxParser{tokens, error_strings, output, current, line, inited, loaded, valid}
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
        self.line = 1;
        self.valid = true;
        self.inited = true;
        self.loaded = false;
    }

    pub fn parse(&mut self) -> Result<Expression, Vec<String>> {
        if self.inited && !self.loaded {
            self.loaded = true;
            let result = self.expression();
            match result {
                Ok(b) => self.output = *b,
                Err(()) => self.valid = false,
            }
        }

        if !self.loaded { Err(vec![String::from("Error: parser has not recieved input.")]) }
        else if self.valid { Ok(self.output.clone()) }
        else { Err(self.error_strings.clone()) }
    }

    fn expression(&mut self) -> Result<Box<Expression>, ()> {
        let t = self.consume()?;
        Ok(self.equality(t)?)
    }

    fn equality(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.comparison(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::BangEqual, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::NotEqual,
                        self.comparison(right)?,
                    )
                },
                Some(Token { data: TokenData::EqualEqual, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Equal,
                        self.comparison(right)?,
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn comparison(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.term(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Less, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Less,
                        self.term(right)?,
                    )
                },
                Some(Token { data: TokenData::LessEqual, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::LessEqual,
                        self.term(right)?,
                    )
                },
                Some(Token { data: TokenData::Greater, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Greater,
                        self.term(right)?,
                    )
                },
                Some(Token { data: TokenData::GreaterEqual, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::GreaterEqual,
                        self.term(right)?,
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn term(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.factor(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Minus, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Subtract,
                        self.factor(right)?,
                    )
                },
                Some(Token { data: TokenData::Plus, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Add,
                        self.factor(right)?,
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn factor(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.unary(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Percent, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Modulo,
                        self.unary(right)?,
                    );
                },
                Some(Token { data: TokenData::Slash, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Divide,
                        self.unary(right)?,
                    );
                },
                Some(Token { data: TokenData::Star, line: _ }) => {
                    self.consume()?;
                    let right = self.consume()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Multiply,
                        self.unary(right)?,
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
                let arg = self.consume()?;
                let u = self.unary(arg)?;
                Ok(Expression::boxed_not(u))
            },
            TokenData::Minus => {
                let arg = self.consume()?;
                let u = self.unary(arg)?;
                Ok(Expression::boxed_negative(u))
            },
            _ => Ok(self.primary(t)?),
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
                let e = self.expression()?;
                if !self.is_at_end() {
                    if let Some(Token{ data: TokenData::RightParen, line: _ }) = self.peek() {
                        self.consume()?;
                        Ok(Expression::boxed_grouping(e))
                    } else {
                        self.add_error("Missing close parenthesis.");
                        Err(())
                    }
                } else {
                    self.add_error("Ran out of tokens unexpectedly. (This likely indicates a scanner bug.)");
                    Err(())
                }
                
            },

            TokenData::EndOfFile => {
                self.add_error("Expected an expression; reached end of file.");
                Err(())
            },
            _ => {
                self.add_error(&format!("Expected an expression; recieved {:?}.", t.data));
                Err(())
            },
        }
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            let prev = self.consume().expect("While loop condition should guarantee consume()");
            if let TokenData::Semicolon = prev.data { break; }
            let next = self.peek();
            if let Some(t) = next {
                match t.data {
                    TokenData::Class |
                    TokenData::Fun |
                    TokenData::Var |
                    TokenData::For |
                    TokenData::If |
                    TokenData::While |
                    TokenData::Print |
                    TokenData::Return => break,
                    _ => (),
                }
            };
        }
    }

    // Can be safely called while at end of file.
    fn peek(&self) -> Option<Token> {
        if !self.is_at_end() {
            let t = self.tokens[self.current].clone();
            Some(t)
        }
        else { None }
    }

    // Will trigger error detection at end of file.
    fn consume(&mut self) -> Result<Token, ()> {
        if !self.is_at_end() {
            let t = self.tokens[self.current].clone();
            self.line = t.line;
            self.current += 1;
            Ok(t)
        }
        else { 
            self.add_error("Ran out of tokens unexpectedly. (This likely indicates a scanner bug.)");
            Err(())
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn add_error(&mut self, message: &str) {
        self.error_strings.push(lox_error::new_error_string(self.line, message));
        self.valid = false;
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