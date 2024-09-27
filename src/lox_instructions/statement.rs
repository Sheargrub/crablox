use crate::lox_instructions::expression::Expression;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Statement {
    Expr(Expression),
    Print(Expression),
}