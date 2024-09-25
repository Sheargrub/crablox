pub mod lox_token;

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

            while !self.is_at_end() {
                let c = self.source[self.current];
                self.current += 1;

                match c {
                    '(' => self.add_token(TokenData::LeftParen),
                    ')' => self.add_token(TokenData::RightParen),
                    '{' => self.add_token(TokenData::LeftBrace),
                    '}' => self.add_token(TokenData::RightBrace),
                    ',' => self.add_token(TokenData::Comma),
                    '.' => self.add_token(TokenData::Dot),
                    ';' => self.add_token(TokenData::Semicolon),
                    '+' => self.add_token(TokenData::Plus),
                    '-' => self.add_token(TokenData::Minus),
                    '*' => self.add_token(TokenData::Star),
                    // Slash needs extra handling for comments - see below
                    '%' => self.add_token(TokenData::Percent),

                    '!' => {
                        if self.match_char('=') { self.add_token(TokenData::BangEqual) }
                        else                    { self.add_token(TokenData::Bang) }
                    }
                    '=' => {
                        if self.match_char('=') { self.add_token(TokenData::EqualEqual) }
                        else                    { self.add_token(TokenData::Equal) }
                    }
                    '<' => {
                        if self.match_char('=') { self.add_token(TokenData::LessEqual) }
                        else                    { self.add_token(TokenData::Less) }
                    }
                    '>' => {
                        if self.match_char('=') { self.add_token(TokenData::GreaterEqual) }
                        else                    { self.add_token(TokenData::Greater) }
                    }

                    '/' => self.add_token(TokenData::Slash), // TODO: comment handling

                    other => self.add_error(&format!("Unexpected character {c}")),
                };

                self.start = self.current;
            }

            self.add_token(TokenData::EndOfFile);
            Ok(self.tokens.clone()) // TODO: check for validity, return error string if not
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    } 

    // If the next character matches c, consumes it and returns true.
    // Otherwise, returns false.
    fn match_char(&mut self, c: char) -> bool {
        if self.is_at_end() { false }
        else if self.source[self.current] != c { false }
        else {
            self.current += 1;
            true
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

    fn test_scan_generic(in_string: &str, expected_tokens: Vec<Token>) {
        let mut scanner = LoxScanner::new(in_string);
        let tokens = scanner.scan_tokens().expect("Unknown scanning failure.");

        let mut i = 0;
        for expected in expected_tokens {
            assert_eq!(
                tokens[i],
                expected
            );
            i += 1;
        };
    }

    #[test]
    fn test_scan_delimiters () {
        let expected_tokens = vec![
            Token::new(LeftParen, 1),
            Token::new(RightParen, 1),
            Token::new(LeftBrace, 1),
            Token::new(RightBrace, 1),
            Token::new(Dot, 1),
            Token::new(Comma, 1),
            Token::new(Semicolon, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic("(){}.,;", expected_tokens);
    }

    #[test]
    fn test_scan_math_ops () {
        let expected_tokens = vec![
            Token::new(Plus, 1),
            Token::new(Minus, 1),
            Token::new(Star, 1),
            Token::new(Slash, 1),
            Token::new(Percent, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic("+-*/%", expected_tokens);
    }

    #[test]
    fn test_scan_comparators () {
        let expected_tokens = vec![
            Token::new(Less, 1),
            Token::new(Greater, 1),
            Token::new(LessEqual, 1),
            Token::new(GreaterEqual, 1),
            Token::new(BangEqual, 1),
            Token::new(EqualEqual, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic("<><=>=!===", expected_tokens);
    }
}