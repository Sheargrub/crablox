pub mod node;

use crate::lox_parser::expression as lox_expression;
use lox_expression::node as lox_node;
use lox_node::*;

#[derive(Debug)]
#[derive(PartialEq)]
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

    pub fn new_number(n: f64) -> Expression {
        LExp(Number(n))
    }
    pub fn new_string(s: &str) -> Expression {
        LExp(StringData(String::from(s)))
    }
    pub fn new_bool(b: bool) -> Expression {
        if b { LExp(True) }
        else { LExp(False) }
    }
    pub fn new_nil() -> Expression {
        LExp(Nil)
    }

    pub fn new_negative(e: Box<Expression>) -> Expression {
        UExp(Negative(e))
    }
    pub fn new_not(e: Box<Expression>) -> Expression {
        UExp(Not(e))
    }

    pub fn new_binary(
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>
    ) -> Expression {
        BExp(Binary{ left, operator, right })
    }

    pub fn new_grouping(e: Box<Expression>) -> Expression {
        Grouping(e)
    }
}

