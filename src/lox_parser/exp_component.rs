use crate::lox_parser::expression::Expression;

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    StringData(String),
    True,
    False,
    Nil,
}

#[derive(Debug)]
pub enum Unary {
    Negative(Box<Expression>),
    Not(Box<Expression>),
}

#[derive(Debug)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: BinaryOp,
    pub right: Box<Expression>,
}

#[derive(Debug)]
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