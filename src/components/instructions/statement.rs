use crate::components as lox;
use lox::instructions::expression::Expression;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Statement {
    Decl(String, Box<Expression>),
    Expr(Box<Expression>),
    Print(Box<Expression>),
    Block(Vec<Box<Statement>>),
    Return(Box<Expression>, usize), // usize is line number for error reporting
    If(Box<Expression>, Box<Statement>, Option<Box<Statement>>),
    While(Box<Expression>, Box<Statement>),
    Fun(String, Vec<String>, Vec<Box<Statement>>), // The strings are unwrapped identifiers
    Class(String, String, Vec<Box<Statement>>), // The statements should all be function declarations
}