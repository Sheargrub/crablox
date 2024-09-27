use crate::components as lox;
use lox::instructions::expression::Expression;
use lox::instructions::node::Identifier;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Statement {
    Decl(Identifier, Box<Expression>),
    Expr(Box<Expression>),
    Print(Box<Expression>),
}