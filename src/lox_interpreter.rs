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

pub fn evaluate(e: Expression) -> Literal {
    match e {
        LExp(l) => l,
        UExp(u) => evaluate_unary(u),
        BExp(b) => evaluate_binary(b),
        Grouping(boxed_exp) => evaluate(*boxed_exp),
    }
}

fn evaluate_unary(u: Unary) -> Literal {
    match u {
        Unary::Negative(expr) => {
            match *expr {
                LExp(l) => Number(-get_number(l)),
                other => {
                    let inner = Expression::boxed_literal(evaluate(other));
                    evaluate_unary(Unary::Negative(inner))
                },
            }
        },
        Unary::Not(expr) => {
            if let LExp(l) = *expr { Boolean(is_truthful(l)) }
            else {
                let inner = Expression::boxed_literal(evaluate(*expr));
                evaluate_unary(Unary::Not(inner))
            }
        },
    }
}

fn evaluate_binary(b: Binary) -> Literal {
    use lox_node::BinaryOp::*;
    
    let left = evaluate(*b.left);
    let right = evaluate(*b.right);

    match b.operator {
        Add => {
            match (left, right) {
                (Number(m), Number(n)) => Number(m + n),
                (StringData(s), StringData(t)) => StringData(format!("{}{}", s, t)),
                _ => panic!("Unhandled exception: mismatched operands to +."),
            }
        }
        Subtract => Number(get_number(left) - get_number(right)),
        Multiply => Number(get_number(left) * get_number(right)),
        Divide => Number(get_number(left) / get_number(right)),
        Modulo => Number(get_number(left) % get_number(right)),

        Less => Boolean(get_number(left) < get_number(right)),
        LessEqual => Boolean(get_number(left) <= get_number(right)),
        Greater => Boolean(get_number(left) > get_number(right)),
        GreaterEqual => Boolean(get_number(left) >= get_number(right)),

        Equal => Boolean(left == right),
        NotEqual => Boolean(left != right),
    }
}

fn is_truthful(l: Literal) -> bool {
    match l {
        Boolean(false) => false,
        Nil => false,
        _ => true,
    }
}

fn get_number(l: Literal) -> f64 {
    if let Number(n) = l { n }
    else { panic!("Unhandled exception: attempted to cast to Number") }
}