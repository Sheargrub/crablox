use crate::components as lox;
use lox::instructions::{statement, expression, node};
use statement::Statement;
use expression::Expression;
use expression::Expression::*;
use node::*;
use node::Literal::*;

pub struct LoxInterpreter {

}

impl LoxInterpreter {
    pub fn new() -> LoxInterpreter {
        LoxInterpreter{}
    }

    pub fn interpret(&self, program: Vec<Statement>) {
        for s in program {
            let result = self.evaluate_stmt(s);
            if let Err(e) = result {
                println!("{}", e);
                break;
            }
        }
    }

    pub fn evaluate_decl(&self, s: Statement) -> Result<(), String> {
        use Statement::*;
        match s {
            Decl(_, _) => {
                // TODO
                Ok(())
            },
            _ => self.evaluate_stmt(s)
        }
    }

    pub fn evaluate_stmt(&self, s: Statement) -> Result<(), String> {
        use Statement::*;
        match s {
            Print(e) => {
                // TODO: I'd like to get a proper std_out working
                println!("{}", self.evaluate_expr(*e)?);
                Ok(())
            },
            Expr(e) => {
                self.evaluate_expr(*e)?;
                Ok(())
            },
            Decl(_, _) => {
                Err(String::from("Cannot use a declaration within another statement."))
            }
        }
    }

    pub fn evaluate_expr(&self, e: Expression) -> Result<Literal, String> {
        match e {
            LExp(l) => Ok(l),
            UExp(u) => self.evaluate_expr_unary(u),
            BExp(b) => self.evaluate_expr_binary(b),
            Grouping(boxed_exp) => self.evaluate_expr(*boxed_exp),
        }
    }

    fn evaluate_expr_unary(&self, u: Unary) -> Result<Literal, String> {
        match u {
            Unary::Negative(expr) => {
                match *expr {
                    LExp(l) => Ok(Number(-get_number(l)?)),
                    other => {
                        let inner = Expression::boxed_literal(self.evaluate_expr(other)?);
                        self.evaluate_expr_unary(Unary::Negative(inner))
                    },
                }
            },
            Unary::Not(expr) => {
                if let LExp(l) = *expr { Ok(Boolean(is_truthful(l))) }
                else {
                    let inner = Expression::boxed_literal(self.evaluate_expr(*expr)?);
                    self.evaluate_expr_unary(Unary::Not(inner))
                }
            },
        }
    }

    fn evaluate_expr_binary(&self, b: Binary) -> Result<Literal, String> {
        use node::BinaryOp::*;
        
        let left = self.evaluate_expr(*b.left)?;
        let right = self.evaluate_expr(*b.right)?;

        match b.operator {
            Add => {
                let error_str = format!("Attempted to add mismatched operands {} and {}.", left, right);
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
    else { Err(format!("Attempted to use literal {} in place of a Number.", l)) }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn string_to_expr(s: &str) -> Expression {
        let mut parser = lox::parser::LoxParser::new();
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
        let intp = LoxInterpreter::new();
        let result = intp.evaluate_expr(expr).expect("Evaluation error");
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