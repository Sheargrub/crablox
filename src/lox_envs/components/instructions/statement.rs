use crate::lox_envs::components as lox;
use lox::instructions::expression::Expression;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Statement {
    Expr(Box<Expression>),
    Print(Box<Expression>),
}