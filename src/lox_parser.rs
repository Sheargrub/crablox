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

    fn expression(&mut self) -> Expression {
        let t = self.consume_unsafe();
        *self.equality(t)
    }

    fn equality(&mut self, t: Token) -> Box<Expression> {
        let e = self.comparison(t);
        match self.peek() {
            Some(Token { data: TokenData::BangEqual, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::NotEqual,
                    self.equality(right_t),
                ))
            },
            Some(Token { data: TokenData::EqualEqual, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::NotEqual,
                    self.equality(right_t),
                ))
            },
            _ => e,
        }
    }

    fn comparison(&mut self, t: Token) -> Box<Expression> {
        let e = self.term(t);
        match self.peek() {
            Some(Token { data: TokenData::Less, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Less,
                    self.comparison(right_t),
                ))
            },
            Some(Token { data: TokenData::LessEqual, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::LessEqual,
                    self.comparison(right_t),
                ))
            },
            Some(Token { data: TokenData::Greater, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Greater,
                    self.comparison(right_t),
                ))
            },
            Some(Token { data: TokenData::GreaterEqual, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::GreaterEqual,
                    self.comparison(right_t),
                ))
            },
            _ => e,
        }
    }

    fn term(&mut self, t: Token) -> Box<Expression> {
        let e = self.factor(t);
        match self.peek() {
            Some(Token { data: TokenData::Minus, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Subtract,
                    self.term(right_t),
                ))
            },
            Some(Token { data: TokenData::Plus, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Add,
                    self.term(right_t),
                ))
            },
            _ => e,
        }
    }

    fn factor(&mut self, t: Token) -> Box<Expression> {
        let e = self.unary(t);
        match self.peek() {
            Some(Token { data: TokenData::Percent, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Modulo,
                    self.factor(right_t),
                ))
            },
            Some(Token { data: TokenData::Slash, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Divide,
                    self.factor(right_t),
                ))
            },
            Some(Token { data: TokenData::Star, line: _ }) => {
                self.consume_unsafe();
                let right_t = self.consume_unsafe();
                Box::new(Expression::new_binary(
                    e,
                    BinaryOp::Multiply,
                    self.factor(right_t),
                ))
            },
            _ => e,
        }
    }

    fn unary(&mut self, t: Token) -> Box<Expression> {
        match t.data {
            TokenData::Bang => {
                let arg = self.consume_unsafe();
                let u = self.unary(arg);
                Box::new(Expression::new_not(u))
            }
            TokenData::Minus => {
                let arg = self.consume_unsafe();
                let u = self.unary(arg);
                Box::new(Expression::new_negative(u))
            }
            _ => self.primary(t)
        }
    }

    fn primary(&mut self, t: Token) -> Box<Expression> {
        Box::new(match t.data {
            TokenData::Number(n) => Expression::new_number(n),
            TokenData::StringData(s) => Expression::new_string(&s),
            TokenData::True => Expression::new_bool(true),
            TokenData::False => Expression::new_bool(false),
            TokenData::Nil => Expression::new_nil(),

            TokenData::LeftParen => {
                let e = Box::new(self.expression());
                if !self.is_at_end() {
                    if let Some(Token{ data: TokenData::RightParen, line: _ }) = self.peek() {
                        self.consume_unsafe();
                    } else {
                        panic!("Unhandled exception: missing close paren.")
                    }
                } else {
                    panic!("Unhandled exception: missing close paren.")
                }
                Expression::new_grouping(e)
            },

            TokenData::EndOfFile => panic!("Unhandled exception: reached end of file unexpectedly."),
            _ => {
                panic!("Unhandled exception: reached impossible state while parsing tokens.");
            },
        })
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
    
    #[test]
    fn test_expression_primary() {
        
        let tokens = vec![
            Token::new(TokenData::StringData(String::from("Hello world!")), 1),
            Token::new(TokenData::EndOfFile, 1),
        ];
        let mut parser = LoxParser{
            tokens,
            error_strings: Vec::new(),
            current: 0,
            inited: true,
            valid: true,
        };

        let t = parser.consume().expect("Parser failed to consume a token.");
        let result = parser.primary(t);
        
        let expected = Expression::new_string("Hello world!");

        assert_eq!(
            expected,
            *result,
            "Expected to recieve left side; recieved right."
        );
    }

    #[test]
    fn test_expression_unary() {
        
        let tokens = vec![
            Token::new(TokenData::Bang, 1),
            Token::new(TokenData::Bang, 1),
            Token::new(TokenData::False, 1),
            Token::new(TokenData::EndOfFile, 1),
        ];
        let mut parser = LoxParser{
            tokens,
            error_strings: Vec::new(),
            current: 0,
            inited: true,
            valid: true,
        };

        let t = parser.consume().expect("Parser failed to consume a token.");
        let result = parser.unary(t);
        
        let expected = Expression::new_not(Box::new(Expression::new_not(Box::new(Expression::new_bool(false)))));

        assert_eq!(
            expected,
            *result,
            "Expected to recieve left side; recieved right."
        );
    }


}