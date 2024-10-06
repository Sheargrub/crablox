mod token;
mod scanner;

use crate::components as lox;

use lox::parser::scanner as lox_scanner;
use lox_scanner::*;

use lox::parser::token as lox_token;
use lox_token::*;

use lox::instructions::{statement as lox_statement, expression as lox_expression, node as lox_node};
use lox_statement::Statement;
use lox_expression::Expression;
use lox_node::*;

pub struct LoxParser {
    tokens: Vec<Token>,
    error_strings: Vec<String>,
    output: Vec<Statement>,
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
        let output = Vec::new();
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
        if self.loaded { 
            self.error_strings = Vec::new();
            self.output = Vec::new();
        }
        self.current = 0;
        self.line = 1;
        self.valid = true;
        self.inited = true;
        self.loaded = false;
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, Vec<String>> {
        if self.inited && !self.loaded {
            self.loaded = true;

            while !self.is_at_end() && self.consume(TokenData::EndOfFile) == None {
                let r = self.statement();
                if let Ok(st) = r {
                    self.output.push(st);
                }
                else {
                    self.valid = false;
                    self.synchronize();
                }
            }
        }

        if !self.loaded { Err(vec![String::from("Error: parser has not recieved input.")]) }
        else if self.valid { Ok(self.output.clone()) }
        else { Err(self.error_strings.clone()) }
    }

    fn statement(&mut self) -> Result<Statement, ()> {
        if self.consume(TokenData::Var).is_some() {
            self.stmt_decl()
        } else if let Some(t) = self.peek() {
            self.stmt_nestable()
        } else {
            self.add_error("Attempted to read a statement while no tokens were present.");
            Err(())
        }
    }

    fn stmt_decl(&mut self) -> Result<Statement, ()> {
        let next = self.advance()?;

        if let TokenData::Identifier(id) = next.data {
            let mut expr = Expression::boxed_nil();
            if !self.is_at_end() && self.consume(TokenData::Equal).is_some() {
                expr = self.expression()?;
            }
            let d = Statement::Decl(id, expr);
            self.pass_semicolon();
            Ok(d)
        }
        
        else {
            self.add_error("Expected variable name after 'var'.");
            Err(())
        }
    }

    fn stmt_nestable(&mut self) -> Result<Statement, ()> {
        let next = self.peek();
        if let Some(t) = next {
            match t.data {
                TokenData::LeftBrace => {
                    self.advance().expect("If-let condition should guarantee advance()");
                    Ok(Statement::Block(self.block()?))
                }
                TokenData::Print => {
                    self.advance().expect("If-let condition should guarantee advance()");
                    let e = Statement::Print(self.expression()?);
                    self.pass_semicolon();
                    Ok(e)
                }
                TokenData::If => {
                    self.advance().expect("If-let condition should guarantee advance()");
                    if !self.consume(TokenData::LeftParen).is_some() {
                        self.add_error("Expected '(' after if statement.");
                        return Err(());
                    };
                    let condition = self.expression()?;
                    if !self.consume(TokenData::RightParen).is_some() {
                        self.add_error("Expected ')' after if condition.");
                        return Err(());
                    };
                    let then_branch = Box::new(self.stmt_nestable()?);

                    if self.consume(TokenData::Else).is_some() {
                        let else_branch = Some(Box::new(self.stmt_nestable()?));
                        Ok(Statement::If(condition, then_branch, else_branch))
                    } else {
                        Ok(Statement::If(condition, then_branch, None))
                    }
                }
                TokenData::While => {
                    self.advance().expect("If-let condition should guarantee advance()");
                    if !self.consume(TokenData::LeftParen).is_some() {
                        self.add_error("Expected '(' after if statement.");
                        return Err(());
                    };
                    let condition = self.expression()?;
                    if !self.consume(TokenData::RightParen).is_some() {
                        self.add_error("Expected ')' after if condition.");
                        return Err(());
                    };
                    let body = Box::new(self.stmt_nestable()?);
                    Ok(Statement::While(condition, body))
                }
                TokenData::For => {
                    self.advance().expect("If-let condition should guarantee advance()");
                    if !self.consume(TokenData::LeftParen).is_some() {
                        self.add_error("Expected '(' after if statement.");
                        return Err(());
                    };
                    let mut for_vec: Vec<Box<Statement>> = Vec::new();
    
                    // Initializer: placed directly at the front of the block
                    if !self.consume(TokenData::Semicolon).is_some() {
                        if self.consume(TokenData::Var).is_some() {
                            for_vec.push(Box::new(self.stmt_decl()?)); // passes semicolon implicitly
                        } else {
                            for_vec.push(Box::new(Statement::Expr(self.expression()?)));
                            self.pass_semicolon();
                        }
                    }
                    
                    let cond = self.expression()?;
                    self.pass_semicolon();
                    
                    let mut while_body: Vec<Box<Statement>> = Vec::new();
                    if !self.consume(TokenData::RightParen).is_some() {
                        let incr = Box::new(Statement::Expr(self.expression()?));
                        if !self.consume(TokenData::RightParen).is_some() {
                            self.add_error("Expected ')' after for clauses.");
                            return Err(());
                        };
                        while_body.push(Box::new(self.stmt_nestable()?)); // for loop body
                        while_body.push(incr);
                    } else {
                        while_body.push(Box::new(self.stmt_nestable()?));
                    }

                    for_vec.push(
                        Box::new(Statement::While(
                            cond,
                            Box::new(Statement::Block(while_body)),
                        ))
                    );

                    Ok(Statement::Block(for_vec))
                }
                _ => { // Expression statement
                    let e = Statement::Expr(self.expression()?);
                    self.pass_semicolon();
                    Ok(e)
                },
            }
        } else {
            self.add_error("Attempted to read a statement while no tokens were present.");
            Err(())
        }
    }

    fn block(&mut self) -> Result<Vec<Box<Statement>>, ()> {
        let mut block = Vec::new();
        let mut block_valid = true;
        while !self.is_at_end() && self.consume(TokenData::RightBrace) == None {
            let r = self.statement();
            if let Ok(st) = r { block.push(Box::new(st)); }
            else {
                block_valid = false;
                self.synchronize();
            }
        }
        if block_valid {Ok(block)}
        else { Err(()) }
    }

    fn expression(&mut self) -> Result<Box<Expression>, ()> {
        let t = self.advance()?;
        Ok(self.assignment(t)?)
    }

    fn assignment(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        if let TokenData::Identifier(ref id) = t.data {
            if self.consume(TokenData::Equal).is_some() {
                Ok(Expression::boxed_assignment(id, self.expression()?))
            } else {
                self.logic_or(t)
            }
        }
        else {
            self.logic_or(t)
        }
    }

    fn logic_or(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.logic_and(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::Or, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_logical(
                        e,
                        LogicOp::Or,
                        self.logic_and(right)?,
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn logic_and(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.equality(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::And, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_logical(
                        e,
                        LogicOp::And,
                        self.equality(right)?,
                    )
                },
                _ => break,
            };
        }
        Ok(e)
    }

    fn equality(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let mut e = self.comparison(t)?;
        loop {
            match self.peek() {
                Some(Token { data: TokenData::BangEqual, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::NotEqual,
                        self.comparison(right)?,
                    )
                },
                Some(Token { data: TokenData::EqualEqual, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
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
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Less,
                        self.term(right)?,
                    )
                },
                Some(Token { data: TokenData::LessEqual, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::LessEqual,
                        self.term(right)?,
                    )
                },
                Some(Token { data: TokenData::Greater, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Greater,
                        self.term(right)?,
                    )
                },
                Some(Token { data: TokenData::GreaterEqual, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
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
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Subtract,
                        self.factor(right)?,
                    )
                },
                Some(Token { data: TokenData::Plus, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
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
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Modulo,
                        self.unary(right)?,
                    );
                },
                Some(Token { data: TokenData::Slash, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
                    e = Expression::boxed_binary(
                        e,
                        BinaryOp::Divide,
                        self.unary(right)?,
                    );
                },
                Some(Token { data: TokenData::Star, line: _ }) => {
                    self.advance()?;
                    let right = self.advance()?;
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
                let arg = self.advance()?;
                let u = self.unary(arg)?;
                Ok(Expression::boxed_unary(UnaryOp::Not, u))
            },
            TokenData::Minus => {
                let arg = self.advance()?;
                let u = self.unary(arg)?;
                Ok(Expression::boxed_unary(UnaryOp::Negative, u))
            },
            _ => Ok(self.call(t)?),
        }
    }

    fn call(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        let line = t.line;
        let mut expr = self.primary(t)?;

        while self.consume(TokenData::LeftParen).is_some() {
            if self.consume(TokenData::RightParen).is_some() {
                expr = Expression::boxed_call(expr, vec![], line);
                continue;
            }

            let mut args: Vec<Box<Expression>> = Vec::new();
            loop {
                args.push(self.expression()?);
                if !self.consume(TokenData::Comma).is_some() { break; }
            }

            if self.consume(TokenData::RightParen).is_some() {
                expr = Expression::boxed_call(expr, args, line);
            } else {
                self.add_error("Unexpectedly reached end of file while parsing arguments.");
                return Err(());
            }
        }

        Ok(expr)
    }

    fn primary(&mut self, t: Token) -> Result<Box<Expression>, ()> {
        match t.data {
            TokenData::Identifier(id) => Ok(Expression::boxed_identifier(&id)),
            TokenData::Number(n) => Ok(Expression::boxed_number(n)),
            TokenData::StringData(s) => Ok(Expression::boxed_string(&s)),
            TokenData::True => Ok(Expression::boxed_boolean(true)),
            TokenData::False => Ok(Expression::boxed_boolean(false)),
            TokenData::Nil => Ok(Expression::boxed_nil()),

            TokenData::LeftParen => {
                let e = self.expression()?;
                if !self.is_at_end() {
                    if let Some(Token{ data: TokenData::RightParen, line: _ }) = self.peek() {
                        self.advance()?;
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
            let prev = self.advance().expect("While loop condition should guarantee advance()");
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

    // Will consume the next token iff it is of the same type as error_str.
    fn consume(&mut self, token_type: TokenData) -> Option<Token> {
        use std::mem::discriminant;
        let next = self.peek();

        if let Some(t) = next {
            if discriminant(&token_type) == discriminant(&t.data) {
                return Some(self.advance().expect("if-let condition should guarantee advance()"));
            }
        }
        None
    }

    // Will trigger error detection at end of file.
    fn advance(&mut self) -> Result<Token, ()> {
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

    fn pass_semicolon(&mut self) {
        let sc = self.consume(TokenData::Semicolon);
        if sc == None {
            self.add_error("Expected ';' at end of statement.");
            self.valid = false;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn add_error(&mut self, message: &str) {
        self.error_strings.push(lox::error::new_error_string(self.line, message));
        self.valid = false;
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_expression_generic(test_str: &str, expected: Box<Expression>) {
        let mut parser = LoxParser::new();
        parser.load_string(test_str).expect("Error while scanning input string");
        let output = parser.expression();

        if let Ok(result) = output {
            assert_eq!(
                expected,
                result,
                "Expected to recieve left side; recieved right."
            );
        } else {
            panic!("Error while parsing expression: {:?}", parser.error_strings);
        }
        
    }

    fn test_statement_generic(test_str: &str, expected: Statement) {
        let mut parser = LoxParser::new();
        parser.load_string(test_str).expect("Error while scanning input string");
        let output = parser.statement();

        if let Ok(result) = output {
            assert_eq!(
                expected,
                result,
                "Expected to recieve left side; recieved right."
            );
        } else {
            panic!("Error while parsing statement: {:?}", parser.error_strings);
        }
    }

    fn test_program_generic(test_str: &str, expected: Vec<Statement>) {
        let mut parser = LoxParser::new();
        parser.load_string(test_str).expect("Error while scanning input string");
        let output = parser.parse();

        if let Ok(result) = output {
            assert_eq!(
                expected,
                result,
                "Expected to recieve left side; recieved right."
            );
        } else {
            panic!("Error while parsing statement: {:?}", parser.error_strings);
        }
    }
    
    mod atomic_expressions {
        use super::*;

        #[test]
        fn test_expression_identifier() {
            let test_str = "my_var";
            let expected = Expression::boxed_identifier("my_var");
            test_expression_generic(test_str, expected);
        }
        
        #[test]
        fn test_expression_string() {
            let test_str = "\"Hello world!\"";
            let expected = Expression::boxed_string("Hello world!");
            test_expression_generic(test_str, expected);
        }

        #[test]
        #[should_panic]
        fn test_expression_unterminated_string() {
            let test_str = "\"Hello world!";
            let expected = Expression::boxed_identifier("my_var");
            test_expression_generic(test_str, expected);
        }
    }

    mod unary_expressions {
        use super::*;

        #[test]
        fn test_expression_unary_not() {
            let test_str = "!!false";
            let expected = Expression::boxed_unary(UnaryOp::Not, Expression::boxed_unary(UnaryOp::Not, Expression::boxed_boolean(false)));
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_unary_negative() {
            let test_str = "-4.3";
            let expected = Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(4.3));
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

    mod logical_expressions {
        use super::*;

        #[test]
        fn test_expression_and() {
            let test_str = "false and true";
            let expected = Expression::boxed_logical(
                Expression::boxed_boolean(false),
                LogicOp::And,
                Expression::boxed_boolean(true),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_or() {
            let test_str = "false or true";
            let expected = Expression::boxed_logical(
                Expression::boxed_boolean(false),
                LogicOp::Or,
                Expression::boxed_boolean(true),
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
                        Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(4.0)),
                        BinaryOp::Multiply,
                        Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(5.0)),
                    ),
                ),
                BinaryOp::Subtract,
                Expression::boxed_number(6.0),
            );
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_logic_ops() {
            let test_str = "1 and 2 or 3 and 4 or 5 and 6";
            let expected = Expression::boxed_logical(
                Expression::boxed_logical(
                    Expression::boxed_logical(
                        Expression::boxed_number(1.0),
                        LogicOp::And,
                        Expression::boxed_number(2.0),
                    ),
                    LogicOp::Or,
                    Expression::boxed_logical(
                        Expression::boxed_number(3.0),
                        LogicOp::And,
                        Expression::boxed_number(4.0),
                    ),
                ),
                LogicOp::Or,
                Expression::boxed_logical(
                    Expression::boxed_number(5.0),
                    LogicOp::And,
                    Expression::boxed_number(6.0),
                ),
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
                    Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(4.0)),
                )),
                BinaryOp::Multiply,
                Expression::boxed_grouping(Expression::boxed_binary(
                    Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(5.0)),
                    BinaryOp::Subtract,
                    Expression::boxed_number(6.0),
                )),
            );
            test_expression_generic(test_str, expected);
        }
    }

    mod assignment {
        use super::*;

        #[test]
        fn test_statement_assignment() {
            let test_str = "i = 0;";
            let expected = Statement::Expr(
                Expression::boxed_assignment(
                    "i",
                    Expression::boxed_number(0.0),
                )
            );
            test_statement_generic(test_str, expected);
        }

        #[test]
        fn test_statement_assignment_nested() {
            let test_str = "i = j = 1;";
            let expected = Statement::Expr(
                Expression::boxed_assignment(
                    "i",
                    Expression::boxed_assignment(
                        "j",
                        Expression::boxed_number(1.0),
                    )
                )
            );
            test_statement_generic(test_str, expected);
        }
    }

    mod statements {
        use super::*;

        #[test]
        fn test_statement_expression() {
            let test_str = "3 + -4 * -5 - 6;";
            let expected = Statement::Expr(Expression::boxed_binary(
                Expression::boxed_binary(
                    Expression::boxed_number(3.0),
                    BinaryOp::Add,
                    Expression::boxed_binary(
                        Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(4.0)),
                        BinaryOp::Multiply,
                        Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(5.0)),
                    ),
                ),
                BinaryOp::Subtract,
                Expression::boxed_number(6.0),
            ));
            test_statement_generic(test_str, expected);
        }

        #[test]
        fn test_statement_print() {
            let test_str = "print 3 + -4 * -5 - 6;";
            let expected = Statement::Print(Expression::boxed_binary(
                Expression::boxed_binary(
                    Expression::boxed_number(3.0),
                    BinaryOp::Add,
                    Expression::boxed_binary(
                        Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(4.0)),
                        BinaryOp::Multiply,
                        Expression::boxed_unary(UnaryOp::Negative, Expression::boxed_number(5.0)),
                    ),
                ),
                BinaryOp::Subtract,
                Expression::boxed_number(6.0),
            ));
            test_statement_generic(test_str, expected);
        }

        mod decl {
            use super::*;

            #[test]
            fn test_statement_decl() {
                let test_str = "var i = 0;";
                let expected = Statement::Decl(
                    String::from("i"),
                    Expression::boxed_number(0.0),
                );
                test_statement_generic(test_str, expected);
            }

            #[test]
            #[should_panic]
            fn test_statement_decl_expects_equals() {
                let test_str = "var i 0;";
                let expected = Statement::Decl(
                    String::from("i"),
                    Expression::boxed_number(0.0),
                );
                test_statement_generic(test_str, expected);
            }

            #[test]
            #[should_panic]
            fn test_statement_decl_rejects_statement() {
                let test_str = "var i = print 0;";
                let expected = Statement::Decl(
                    String::from("i"),
                    Expression::boxed_number(0.0),
                );
                test_statement_generic(test_str, expected);
            }
        }

        #[test]
        fn test_statement_while() {
            let test_str = "while (true) print \"This is the program that never ends~\";";
            let expected = Statement::While(
                Expression::boxed_boolean(true),
                Box::new(Statement::Print(
                    Expression::boxed_string("This is the program that never ends~"),
                )),
            );
            test_statement_generic(test_str, expected);
        }

        #[test]
        fn test_statement_for() {
            let test_str = "for (var i = 0; i < 10; i = i * 2) print i;";
            let expected = Statement::Block(vec![
                Box::new(Statement::Decl(
                    String::from("i"),
                    Expression::boxed_number(0.0),
                )),
                Box::new(Statement::While(
                    Expression::boxed_binary(
                        Expression::boxed_identifier("i"),
                        BinaryOp::Less,
                        Expression::boxed_number(10.0),
                    ),
                    Box::new(Statement::Block(vec![
                        Box::new(Statement::Print(
                            Expression::boxed_identifier("i"),
                        )),
                        Box::new(Statement::Expr(Expression::boxed_assignment(
                            "i",
                            Expression::boxed_binary(
                                Expression::boxed_identifier("i"),
                                BinaryOp::Multiply,
                                Expression::boxed_number(2.0),
                            ),
                        ))),
                    ])),
                )),
            ]);
            test_statement_generic(test_str, expected);
        }
    }

    mod programs {
        use super::*;

        #[test]
        fn test_program_hello_world() {
            let source = concat!(
                "var my_var = \"Hello, world!\";\n",
                "print my_var;\n",
            );
            let expected = vec![
                Statement::Decl(
                    String::from("my_var"),
                    Expression::boxed_string("Hello, world!"),
                ),
                Statement::Print(
                    Expression::boxed_identifier("my_var"),
                ),
            ];
            test_program_generic(source, expected);
        }

        #[test]
        fn test_program_assignments() {
            let source = concat!(
                "var i;\n",
                "var j = 2;\n",
                "var k = 3 + 4;\n",
                "i = 3;\n",
                "j = 3 - 1;\n",
                "k = j + k + 3;\n",
                "i = j = k;\n",
            );
            let expected = vec![
                Statement::Decl(
                    String::from("i"),
                    Expression::boxed_nil(),
                ),
                Statement::Decl(
                    String::from("j"),
                    Expression::boxed_number(2.0),
                ),
                Statement::Decl(
                    String::from("k"),
                    Expression::boxed_binary(
                        Expression::boxed_number(3.0),
                        BinaryOp::Add,
                        Expression::boxed_number(4.0),
                    )
                ),
                Statement::Expr(
                    Expression::boxed_assignment(
                        "i",
                        Expression::boxed_number(3.0),
                    )
                ),
                Statement::Expr(
                    Expression::boxed_assignment(
                        "j",
                        Expression::boxed_binary(
                            Expression::boxed_number(3.0),
                            BinaryOp::Subtract,
                            Expression::boxed_number(1.0),
                        )
                    )
                ),
                Statement::Expr(
                    Expression::boxed_assignment(
                        "k",
                        Expression::boxed_binary(
                            Expression::boxed_binary(
                                Expression::boxed_identifier("j"),
                                BinaryOp::Add,
                                Expression::boxed_identifier("k"),
                            ),
                            BinaryOp::Add,
                            Expression::boxed_number(3.0),
                        )
                    )
                ),
                Statement::Expr(
                    Expression::boxed_assignment(
                        "i",
                        Expression::boxed_assignment(
                            "j",
                            Expression::boxed_identifier("k"),
                        )
                    )
                ),
            ];
            test_program_generic(source, expected);
        }

        #[test]
        fn test_program_blocks() {
            let source = concat!(
                "{}\n",
                "var global = 23;\n",
                "{\n",
                "   var local = 3;\n",
                "   { print local; }\n",
                "}\n",
            );
            let expected = vec![
                Statement::Block(vec![]),
                Statement::Decl(
                    String::from("global"),
                    Expression::boxed_number(23.0),
                ),
                Statement::Block(vec![
                    Box::new(Statement::Decl(
                        String::from("local"),
                        Expression::boxed_number(3.0),
                    )),
                    Box::new(Statement::Block(vec![
                        Box::new(Statement::Print(
                            Expression::boxed_identifier("local"),
                        ))
                    ])),
                ]),
            ];
            test_program_generic(source, expected);
        }

        #[test]
        fn test_program_if_else() {
            let source = concat!(
                "if (2 <= 3) print \"Math is working\";\n",
                "var three = 3;\n",
                "if (three == 3) {\n",
                "   print 333;\n",
                "} else {\n",
                "   print 4444;\n",
                "}",
            );
            let expected = vec![
                Statement::If(
                    Expression::boxed_binary(
                        Expression::boxed_number(2.0),
                        BinaryOp::LessEqual,
                        Expression::boxed_number(3.0),
                    ),
                    Box::new(Statement::Print(
                        Expression::boxed_string("Math is working"),
                    )),
                    None,
                ),
                Statement::Decl(
                    String::from("three"),
                    Expression::boxed_number(3.0),
                ),
                Statement::If(
                    Expression::boxed_binary(
                        Expression::boxed_identifier("three"),
                        BinaryOp::Equal,
                        Expression::boxed_number(3.0),
                    ),
                    Box::new(Statement::Block(vec![
                        Box::new(Statement::Print(
                            Expression::boxed_number(333.0),
                        )),
                    ])),
                    Some(Box::new(Statement::Block(vec![
                        Box::new(Statement::Print(
                            Expression::boxed_number(4444.0),
                        )),
                    ]))),
                ),
            ];
            test_program_generic(source, expected);
        }

        #[test]
        fn test_program_functions() {
            let source = concat!(
                "function();\n",
                "argumentative(yes, very);\n",
                "nested(one)(two);\n"
            );
            let expected = vec![
                Statement::Expr(Expression::boxed_call(
                    Expression::boxed_identifier("function"),
                    vec![],
                    1,
                )),
                Statement::Expr(Expression::boxed_call(
                    Expression::boxed_identifier("argumentative"),
                    vec![
                        Expression::boxed_identifier("yes"),
                        Expression::boxed_identifier("very"),
                    ],
                    2,
                )),
                Statement::Expr(Expression::boxed_call(
                    Expression::boxed_call(
                        Expression::boxed_identifier("nested"),
                        vec![
                            Expression::boxed_identifier("one"),
                        ],
                        3,
                    ),
                    vec![
                        Expression::boxed_identifier("two"),
                    ],
                    3,
                )),
            ];
            test_program_generic(source, expected);
        }
    }
}