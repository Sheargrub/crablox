pub mod lox_token;

use std::io;
use crate::lox_scanner::lox_token::*;
use crate::lox_error;

pub struct LoxScanner {
    source : Vec<char>,
    tokens : Vec<Token>,
    errors : Vec<Result<Vec<Token>, String>>,
    start : usize,
    current : usize,
    line : usize,
    inited : bool,
    valid : bool,
}

impl LoxScanner {

    pub fn new(source: &str) -> LoxScanner {
        let source = source.chars().collect();
        let tokens = Vec::new();
        let errors = Vec::new();
        LoxScanner {
            source,
            tokens,
            errors,
            start : 0,
            current : 0,
            line : 1,
            inited : false,
            valid : true,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        if self.inited {
            Err(String::from("Scanner already inited"))
            // TODO: allow repeat calls to this function
        }
        else {
            self.inited = true;

            let num_chars = self.source.len();
            while self.current < num_chars { // TODO: this can panic!
                let c = self.source[self.current];

                match c {
                    '(' => self.add_token(TokenData::LeftParen),
                    ')' => self.add_token(TokenData::RightParen),
                    '{' => self.add_token(TokenData::LeftBrace),
                    '}' => self.add_token(TokenData::RightBrace),
                    ',' => self.add_token(TokenData::Comma),
                    '.' => self.add_token(TokenData::Dot),
                    ';' => self.add_token(TokenData::Semicolon),
                    '-' => self.add_token(TokenData::Minus),
                    '+' => self.add_token(TokenData::Plus),
                    '*' => self.add_token(TokenData::Star),
                    '/' => self.add_token(TokenData::Slash),
                    '%' => self.add_token(TokenData::Percent),
                    other => self.add_error(&format!("Unexpected character {c}")),
                };

                self.current += 1;
                self.start = self.current;
            }

            self.add_token(TokenData::EndOfFile);
            Ok(self.tokens.clone()) // TODO: check for validity, return error string if not
        }
    }

    fn add_token(&mut self, data: TokenData) {
        self.tokens.push(Token{data, line: self.line});
    }

    fn add_error(&mut self, message: &str) {
        self.errors.push(lox_error::generate_err::<Vec<Token>>(self.line, message)); // TODO: icky coupling
        self.valid = false;
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lox_scanner::TokenData::*;

    #[test]
    fn test_scan_delimiters () {
        let mut scanner = LoxScanner::new("(){}.,;");
        let tokens = scanner.scan_tokens().expect("Unknown scanning failure.");

        assert_eq!(
            tokens[0],
            Token::new(LeftParen, 1)
        );
        assert_eq!(
            tokens[1],
            Token::new(RightParen, 1)
        );
        assert_eq!(
            tokens[2],
            Token::new(LeftBrace, 1)
        );
        assert_eq!(
            tokens[3],
            Token::new(RightBrace, 1)
        );
        assert_eq!(
            tokens[4],
            Token::new(Dot, 1)
        );
        assert_eq!(
            tokens[5],
            Token::new(Comma, 1)
        );
        assert_eq!(
            tokens[6],
            Token::new(Semicolon, 1)
        );
    }

    #[test]
    fn test_scan_math_ops () {
        let mut scanner = LoxScanner::new("+-*/%");
        let tokens = scanner.scan_tokens().expect("Unknown scanning failure.");

        assert_eq!(
            tokens[0],
            Token::new(Plus, 1)
        );
        assert_eq!(
            tokens[1],
            Token::new(Minus, 1)
        );
        assert_eq!(
            tokens[2],
            Token::new(Star, 1)
        );
        assert_eq!(
            tokens[3],
            Token::new(Slash, 1)
        );
        assert_eq!(
            tokens[4],
            Token::new(Percent, 1)
        );
    }
}