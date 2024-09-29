use crate::components as lox;
use lox::instructions::{statement, expression, node};
use statement::Statement;
use expression::Expression;
use expression::Expression::*;
use node::*;
use node::Literal::*;
use lox::environment::*;

pub struct LoxInterpreter {
    env: LoxEnvironment,
    output: String,
}

impl LoxInterpreter {
    pub fn new() -> LoxInterpreter {
        LoxInterpreter{ env: LoxEnvironment::new(), output: String::new() }
    }

    pub fn interpret(&mut self, program: Vec<Statement>) -> Result<String, String> {
        self.output = String::new();
        for s in program {
            let result = self.evaluate_stmt(s);
            if let Err(e) = result { return Err(e); }
        }
        self.output.pop();
        Ok(self.output.clone())
    }

    pub fn evaluate_stmt(&mut self, s: Statement) -> Result<(), String> {
        use Statement::*;
        match s {
            Decl(id, expr) => {
                let data = self.evaluate_expr(*expr)?;
                self.env.define(&id, data);
                Ok(())
            },
            Block(v) => {
                self.env.lower_scope();
                for s in v { self.evaluate_stmt(*s)?; }
                self.env.raise_scope().expect("Function structure should guarantee valid scope raise");
                Ok(())
            }
            Print(e) => {
                let text = &format!("{}", self.evaluate_expr(*e)?);
                self.output.push_str(text);
                self.output.push_str("\n");
                Ok(())
            },
            Expr(e) => {
                self.evaluate_expr(*e)?;
                Ok(())
            },
            If(cond, then_branch, else_option) => {
                if is_truthful(self.evaluate_expr(*cond)?) {
                    self.evaluate_stmt(*then_branch)
                } else if let Some(else_branch) = else_option {
                    self.evaluate_stmt(*else_branch)
                } else {
                    Ok(())
                }
            },
        }
    }

    pub fn evaluate_expr(&mut self, e: Expression) -> Result<Literal, String> {
        match e {
            LitExp(lit) => Ok(lit),
            Unary(op, e) => self.evaluate_expr_unary(op, e),
            Binary{left, op, right} => self.evaluate_expr_binary(left, op, right),
            Logical{left, op, right} => self.evaluate_expr_logical(left, op, right),
            Identifier(id) => Ok(self.env.get(&id)?),
            Grouping(boxed_exp) => self.evaluate_expr(*boxed_exp),
            Assignment(id, boxed_exp) => {
                let lit = self.evaluate_expr(*boxed_exp)?;
                Ok(self.env.assign(&id, lit)?)
            }
        }
    }

    fn evaluate_expr_unary(&mut self, op: node::UnaryOp, e: Box<Expression>) -> Result<Literal, String> {
        use node::UnaryOp::*;

        let arg = self.evaluate_expr(*e)?;
        match op {
            Negative => Ok(Number(-get_number(arg)?)),
            Not => Ok(Boolean(is_truthful(arg))),
        }
    }

    fn evaluate_expr_binary(&mut self, left: Box<Expression>, op: node::BinaryOp, right: Box<Expression>) -> Result<Literal, String> {
        use node::BinaryOp::*;
        
        let left = self.evaluate_expr(*left)?;
        let right = self.evaluate_expr(*right)?;

        match op {
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

    fn evaluate_expr_logical(&mut self, left: Box<Expression>, op: node::LogicOp, right: Box<Expression>) -> Result<Literal, String> {
        use node::LogicOp::*;
        
        let left_lit = self.evaluate_expr(*left)?;
        let left_truthful = is_truthful(left_lit.clone()); // TODO: clone operation here is needlessly costly

        match op {
            And => {
                if left_truthful { Ok(self.evaluate_expr(*right)?) }
                else { Ok(left_lit) }
            }
            Or => {
                if !left_truthful { Ok(self.evaluate_expr(*right)?) }
                else { Ok(left_lit) }
            }
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

    fn string_to_program(s: &str) -> Vec<Statement> {
        let mut parser = lox::parser::LoxParser::new();
        parser.load_string(s).expect("Error while scanning input string.");
        parser.parse().expect("Error while parsing expression.")
    }

    fn string_to_expr(s: &str) -> Expression {
        let statements = string_to_program(s);
        if let Statement::Expr(e) = statements[0].clone() { *e }
        else { panic!("Attempted to convert a non-statement to an expression."); }
    }

    fn test_expression_generic(s: &str, expected: Literal) {
        let expr = string_to_expr(s);
        let mut intp = LoxInterpreter::new();
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

    mod logical_expressions {
        use super::*;

        #[test]
        fn test_expression_and() {
            let test_str = "3.0 and 0.0;";
            let expected = Number(0.0);
            test_expression_generic(test_str, expected);
            let test_str = "nil and 0.0;";
            let expected = Nil;
            test_expression_generic(test_str, expected);
        }

        #[test]
        fn test_expression_or() {
            let test_str = "3.0 or 0.0;";
            let expected = Number(3.0);
            test_expression_generic(test_str, expected);
            let test_str = "nil or 0.0;";
            let expected = Number(0.0);
            test_expression_generic(test_str, expected);
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

    mod variables_and_declarations {
        use super::*;

        #[test]
        fn test_variable_declaration() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "var i;",
                "var j = 2;",
            ));
            intp.interpret(program).expect("Error while interpreting program");

            let expected = Nil;
            let result = intp.env.get("i").expect("Failed to retrieve an uninitialized variable");
            assert_eq!(expected, result, "Uninitialized variable should be Nil; instead recieved {}", result);

            let expected = Number(2.0);
            let result = intp.env.get("j").expect("Failed to retrieve an initialized variable");
            assert_eq!(expected, result, "Variable should equal 2; instead recieved {}", result);
        }

        #[test]
        fn test_variable_redeclaration() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "var i = 1;",
                "var i = i + 1;",
            ));
            intp.interpret(program).expect("Error while interpreting program");

            let expected = Number(2.0);
            let result = intp.env.get("i").expect("Failed to retrieve an initialized variable");
            assert_eq!(expected, result, "Variable should equal 2; instead recieved {}", result);
        }

        #[test]
        fn test_variable_assignment() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "var i = 0;",
                "var j;",
                "var k;",
                "i = i + 1;",
                "k = j = 1 + i;",
                "k = k + 1;",
            ));
            intp.interpret(program).expect("Error while interpreting program");
            
            let expected = Number(1.0);
            let result = intp.env.get("i").expect("Failed to retrieve an initialized variable");
            assert_eq!(expected, result, "Variable should equal 1; instead recieved {}", result);

            let expected = Number(2.0);
            let result = intp.env.get("j").expect("Failed to retrieve an initialized variable");
            assert_eq!(expected, result, "Variable should equal 2; instead recieved {}", result);

            let expected = Number(3.0);
            let result = intp.env.get("k").expect("Failed to retrieve an initialized variable");
            assert_eq!(expected, result, "Variable should equal 3; instead recieved {}", result);
        }

        #[test]
        fn test_variable_scope() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "var a = \"global a\";\n",
                "var b = \"global b\";\n",
                "var c = \"global c\";\n",
                "{\n",
                "  var a = \"outer a\";\n",
                "  var b = \"outer b\";\n",
                "  {\n",
                "    var a = \"inner a\";\n",
                "    print a;\n",
                "    print b;\n",
                "    print c;\n",
                "  }\n",
                "  print a;\n",
                "  print b;\n",
                "  print c;\n",
                "}\n",
                "print a;\n",
                "print b;\n",
                "print c;",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = concat!(
                "inner a\n",
                "outer b\n",
                "global c\n",
                "outer a\n",
                "outer b\n",
                "global c\n",
                "global a\n",
                "global b\n",
                "global c",
            );

            assert_eq!(expected, output, "Expected left output; recieved right");
        }

        #[test]
        fn test_if_else() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "if (2 <= 3) print \"Math is working\";\n",
                "var three = 3;\n",
                "if (three == 3) {\n",
                "   print 333;\n",
                "} else {\n",
                "   print 4444;\n",
                "}",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = concat!(
                "Math is working\n",
                "333",
            );

            assert_eq!(expected, output, "Expected left output; recieved right");
        }

        #[test]
        fn test_logical_short_circuiting() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "var i; var j; var k;\n",
                "(i = 1) or (j = 2);\n",
                "print i; print j;\n",
                "j or (k = 2);\n",
                "print k;\n",
                "j and (j = 3);\n",
                "print j;\n",
                "i and (j = 4);\n",
                "print j;\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = concat!(
                "1\n",
                "Nil\n",
                "2\n",
                "Nil\n",
                "4",
            );

            assert_eq!(expected, output, "Expected left output; recieved right");
        }

    }

}