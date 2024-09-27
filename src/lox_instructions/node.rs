use crate::lox_instructions::expression::Expression;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Literal {
    Number(f64),
    StringData(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Unary {
    Negative(Box<Expression>),
    Not(Box<Expression>),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: BinaryOp,
    pub right: Box<Expression>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum BinaryOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}