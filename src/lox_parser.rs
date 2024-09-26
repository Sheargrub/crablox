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
    inited: bool,
    valid: bool,
}

impl LoxParser {

    pub fn new() -> LoxParser {
        let tokens = Vec::new();
        let error_strings = Vec::new();
        let inited = false;
        let valid = true;
        LoxParser{tokens, error_strings, inited, valid}
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
        if self.inited { self.error_strings = Vec::new(); }
        self.valid = true;
        self.inited = true;
    }

    /*
    pub fn parse_string(tokens: <Vec<Token>>) -> Result<Expression, Vec<String>> {
        let mut current = 0;
    }
    */

    /*
    fn expression() -> Expression {
        self.equality()
    }

    fn equality() -> Expression {

    }

    fn comparison() -> Expression {

    }

    fn term() -> Expression {

    }

    fn factor() -> Expression {

    }

    fn unary() -> Expression {

    }

    fn primary() -> Expression {

    }
    */

}