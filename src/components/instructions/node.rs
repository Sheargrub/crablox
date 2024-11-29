use core::fmt;
use crate::components::instructions::callable::Callable;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Literal {
    Number(f64),
    StringData(String),
    Boolean(bool),
    Nil,
    CallLit(Callable),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::StringData(s) => write!(f, "{}", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "Nil"),
            Literal::CallLit(c) => c.fmt(f),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub enum UnaryOp {
    Negative,
    Not,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
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

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub enum LogicOp {
    And,
    Or,
}