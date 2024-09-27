use crate::lox_instructions::expression as lox_expression;
use crate::lox_instructions::node as lox_node;
use lox_node::*;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Expression {
    LExp(Literal),
    UExp(Unary),
    BExp(Binary),
    Grouping(Box<Expression>),
}

use lox_expression::Expression::*;
use lox_node::Literal::*;
use lox_node::Unary::*;
impl Expression {
    pub fn boxed_literal(l: Literal) -> Box<Expression> {
        Box::new(LExp(l))
    }

    pub fn boxed_number(n: f64) -> Box<Expression> {
        Box::new(LExp(Number(n)))
    }
    pub fn boxed_string(s: &str) -> Box<Expression> {
        Box::new(LExp(StringData(String::from(s))))
    }
    pub fn boxed_bool(b: bool) -> Box<Expression> {
        Box::new(LExp(Boolean(b)))
    }
    pub fn boxed_nil() -> Box<Expression> {
        Box::new(LExp(Nil))
    }

    pub fn boxed_negative(e: Box<Expression>) -> Box<Expression> {
        Box::new(UExp(Negative(e)))
    }
    pub fn boxed_not(e: Box<Expression>) -> Box<Expression> {
        Box::new(UExp(Not(e)))
    }

    pub fn boxed_binary(
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>
    ) -> Box<Expression> {
        Box::new(BExp(Binary{ left, operator, right }))
    }

    pub fn boxed_grouping(e: Box<Expression>) -> Box<Expression> {
        Box::new(Grouping(e))
    }
}

