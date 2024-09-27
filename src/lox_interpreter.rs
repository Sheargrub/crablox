use crate::lox_parser::*;
use crate::lox_instructions::{statement as lox_statement, expression as lox_expression, node as lox_node};
use lox_statement::Statement;
use lox_expression::Expression;
use lox_expression::Expression::*;
use lox_node::*;
use lox_node::Literal::*;

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
    else { Err(format!("Attempted to use literal {:?} in place of a Number.", l)) }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn string_to_expr(s: &str) -> Expression {
        use crate::lox_parser::*;
        let mut parser = LoxParser::new();
        parser.load_string(s).expect("Error while scanning input string.");
        let statements = parser.parse().expect("Error while parsing expression.");
        if let Statement::Expr(e) = statements[0].clone() { // TODO: refactor to remove clone call
            *e
        } else {
            panic!("Attempted to convert a non-statement to an expression.");
        }
    }

    fn test_expression_generic(s: &str, expected: Literal) {
        let expr = string_to_expr(s);
        let result = evaluate(expr).expect("Evaluation error");
        assert_eq!(expected, result, "Expected to recieve left side; recieved right.");
    }

    mod utilities {
        use super::*;

        #[test]
        fn test_is_truthful() {
            assert!(is_truthful(Number(0.0)));
            assert!(is_truthful(StringData(String::new())));
            assert!(is_truthful(Boolean(true)));

            assert!(!is_truthful(Boolean(false)));
            assert!(!is_truthful(Nil));
        }

        #[test]
        fn test_get_number() {
            assert_eq!(get_number(Number(43.0)), Ok(43.0));
            if let Err(s) = get_number(Nil) {
                assert!(s.contains("Attempted to use literal"));
            } else {
                panic!("get_number(Nil) failed to return an error");
            }
        }
    }

    mod unary_expressions {
        use super::*;

        #[test]
        fn test_expression_unary_not() {
            test_expression_generic("!!false;", Boolean(false));
        }

        #[test]
        fn test_expression_unary_negative() {
            test_expression_generic("-4.3;", Number(-4.3));
        }
    }

    mod binary_expressions {
        use super::*;

        #[test]
        fn test_expression_modulo() {
            test_expression_generic("5 % 3;", Number(2.0));
        }

        #[test]
        fn test_expression_divide() {
            test_expression_generic("3/5;", Number(0.6));
        }

        #[test]
        fn test_expression_multiply() {
            test_expression_generic("4.1 * 5;", Number(20.5));
        }

        #[test]
        fn test_expression_add() {
            test_expression_generic("4.1 + 5;", Number(9.1));
        }

        #[test]
        fn test_expression_subtract() {
            test_expression_generic("4.1 - 5;", Number(4.1-5.0));
        }

        #[test]
        fn test_expression_less() {
            test_expression_generic("4.1 < 5;", Boolean(true));
        }

        #[test]
        fn test_expression_less_equal() {
            test_expression_generic("4.1 <= 5;", Boolean(true));
        }

        #[test]
        fn test_expression_greater() {
            test_expression_generic("4.1 > 5;", Boolean(false));
        }

        #[test]
        fn test_expression_greater_equal() {
            test_expression_generic("4.1 >= 5;", Boolean(false));
        }

        #[test]
        fn test_expression_equal() {
            test_expression_generic("4.1 == 5;", Boolean(false));
        }

        #[test]
        fn test_expression_not_equal() {
            test_expression_generic("4.1 != 5;", Boolean(true));
        }
    }

    mod compound_expressions {
        use super::*;

        #[test]
        fn test_expression_math_ops() {
            let test_str = "3 + -4 * -5 - 6;";
            test_expression_generic(test_str, Number(17.0));
        }

        #[test]
        fn test_expression_comparison() {
            let test_str = "15 / 5 >= 2 != 1.5 + 1.5 < 2;";
            test_expression_generic(test_str, Boolean(true));
        }

        #[test]
        fn test_expression_grouping() {
            let test_str = "(3 + -4) * (-5 - 6);";
            test_expression_generic(test_str, Number(11.0));
        }
    }
}