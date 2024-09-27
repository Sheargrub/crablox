use crate::lox_parser::*;
use crate::lox_instructions::{expression as lox_expression, node as lox_node};
use lox_expression::Expression;
use lox_expression::Expression::*;
use lox_node::*;
use lox_node::Literal::*;

pub struct LoxInterpreter {
    parser: LoxParser
}

impl LoxInterpreter {
    pub fn new() -> LoxInterpreter {
        let parser = LoxParser::new();
        LoxInterpreter{parser}
    }    
}

pub fn evaluate(e: Expression) -> Result<Literal, String> {
    match e {
        LExp(l) => Ok(l),
        UExp(u) => evaluate_unary(u),
        BExp(b) => evaluate_binary(b),
        Grouping(boxed_exp) => evaluate(*boxed_exp),
    }
}

fn evaluate_unary(u: Unary) -> Result<Literal, String> {
    match u {
        Unary::Negative(expr) => {
            match *expr {
                LExp(l) => Ok(Number(-get_number(l)?)),
                other => {
                    let inner = Expression::boxed_literal(evaluate(other)?);
                    evaluate_unary(Unary::Negative(inner))
                },
            }
        },
        Unary::Not(expr) => {
            if let LExp(l) = *expr { Ok(Boolean(is_truthful(l))) }
            else {
                let inner = Expression::boxed_literal(evaluate(*expr)?);
                evaluate_unary(Unary::Not(inner))
            }
        },
    }
}

fn evaluate_binary(b: Binary) -> Result<Literal, String> {
    use lox_node::BinaryOp::*;
    
    let left = evaluate(*b.left)?;
    let right = evaluate(*b.right)?;

    match b.operator {
        Add => {
            let error_str = format!("Attempted to add mismatched operands {:?} and {:?}.", left, right);
            match (left, right) {
                (Number(m), Number(n)) => Ok(Number(m + n)),
                (StringData(s), StringData(t)) => Ok(StringData(format!("{}{}", s, t))),
                _ => Err(error_str),
            }
        }
        Subtract => Ok(Number(get_number(left)? - get_number(right)?)),
        Multiply => Ok(Number(get_number(left)? * get_number(right)?)),
        Divide => Ok(Number(get_number(left)? / get_number(right)?)),
        Modulo => Ok(Number(get_number(left)? % get_number(right)?)),

        Less => Ok(Boolean(get_number(left)? < get_number(right)?)),
        LessEqual => Ok(Boolean(get_number(left)? <= get_number(right)?)),
        Greater => Ok(Boolean(get_number(left)? > get_number(right)?)),
        GreaterEqual => Ok(Boolean(get_number(left)? >= get_number(right)?)),

        Equal => Ok(Boolean(left == right)),
        NotEqual => Ok(Boolean(left != right)),
    }
}

fn is_truthful(l: Literal) -> bool {
    match l {
        Boolean(false) => false,
        Nil => false,
        _ => true,
    }
}

fn get_number(l: Literal) -> Result<f64, String> {
    if let Number(n) = l { Ok(n) }
    else { Err(format!("Attempted to use literal {:?} in place of a Number", l)) }
}