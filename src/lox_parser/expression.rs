use crate::lox_parser::exp_component::*;

#[derive(Debug)]
pub enum Expression {
    LExp(Literal),
    UExp(Unary),
    BExp(Binary),
}

use crate::lox_parser::exp_component::Literal::*;
use crate::lox_parser::exp_component::Unary::*;
use crate::lox_parser::expression::Expression::*;
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
}



