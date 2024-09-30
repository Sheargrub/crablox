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
    If(Box<Expression>, Box<Statement>, Option<Box<Statement>>),
    While(Box<Expression>, Box<Statement>),
}