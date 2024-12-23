use crate::components as lox;
use lox::instructions::{statement, expression, node, callable, instance};
use statement::Statement;
use expression::Expression;
use expression::Expression::*;
use callable::*;
use node::*;
use node::Literal::*;
use instance::*;
use lox::environment::*;

use std::vec::*;
use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct LoxInterpreter {
    env: LoxEnvironment,
    output: String,
}

impl LoxInterpreter {
    pub fn new() -> LoxInterpreter {
        let mut env = LoxEnvironment::new(); // parens prevent overlap w/ function namespace
        let native_fns = Callable::native_fn_list();
        for f in native_fns.iter() {
            env.define(&f.0, Literal::CallLit(f.1.clone())); // TODO: assess clone call
        }
        LoxInterpreter{ env, output: String::new() }
    }

    // This is mainly a helper function to let functions add variables.
    // Don't love that it's visible as an interface to the end-user, may rework.
    pub fn define_external(&mut self, name: &str, value: Literal) {
        self.env.define(name, value);
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

    // Returns Some if returning a value from a block, None for other valid outcomes
    pub fn evaluate_stmt(&mut self, s: Statement) -> Result<Option<Literal>, String> {
        use Statement::*;
        match s {
            Decl(id, expr) => {
                let data = self.evaluate_expr(*expr)?;
                self.env.define(&id, data);
                Ok(None)
            },
            Block(v) => {
                self.env.lower_scope();
                for s in v {
                    let current = self.evaluate_stmt(*s)?;
                    if let Some(lit) = current {
                        self.env.raise_scope().expect("Block execution structure should guarantee valid scope raise");
                        return Ok(Some(lit));
                    }
                }
                self.env.raise_scope().expect("Block execution structure should guarantee valid scope raise");
                Ok(None)
            }
            Print(e) => {
                //self.env.print_cur_closure();
                let text = &format!("{}", self.evaluate_expr(*e)?);
                self.output.push_str(text);
                self.output.push_str("\n");
                Ok(None)
            },
            Expr(e) => {
                self.evaluate_expr(*e)?;
                Ok(None)
            },
            Return(e, line) => {
                //self.env.print_cur_closure();
                let result = self.evaluate_expr(*e)?;
                Ok(Some(result))
            }
            If(cond, then_branch, else_option) => {
                if is_truthful(self.evaluate_expr(*cond)?) {
                    self.evaluate_stmt(*then_branch)
                } else if let Some(else_branch) = else_option {
                    self.evaluate_stmt(*else_branch)
                } else {
                    Ok(None)
                }
            },
            While(cond, body) => {
                // TODO: clone statements here are horrifically inefficient.
                // Probably need to restructure everything to pass by reference...
                while is_truthful(self.evaluate_expr(*cond.clone())?) {
                    let current = self.evaluate_stmt(*body.clone())?;
                    if let Some(lit) = current {
                        return Ok(Some(lit));
                    }
                }
                Ok(None)
            }
            Fun(name, args, body) => {
                let inner_func = Callable::Function(name.clone(), args.clone(), body.clone(), None, false);
                let closure = self.env.spawn_closure();
                closure.borrow_mut().define(&name, Literal::CallLit(inner_func));
                let data = Callable::Function(name.clone(), args, body, Some(closure), false);
                self.env.define(&name, Literal::CallLit(data));
                Ok(None)
            }
            Class(name, super_name, method_defs) => {
                let mut methods = HashMap::new();
                for stmt in method_defs.clone() {
                    if let Fun(fn_name, args, body) = *stmt {
                        let func = Callable::Function(fn_name.clone(), args, body, Some(self.env.spawn_closure()), fn_name == "init");
                        methods.insert(fn_name.clone(), func);
                    }
                    else { panic!("Found non-function statement while processing methods for class {}.", name); } // should be impossible
                }

                let mut super_class = None;
                if super_name != "" {
                    let super_eval = self.env.get(&super_name)?;
                    if let CallLit(Callable::Class(n, sn, md)) = super_eval {
                        super_class = Some(Box::new(Callable::Class(n, sn, md)));
                    } else {
                        return Err(String::from("Superclass must be a class."));
                    }
                }

                let class = Callable::Class(name.clone(), super_class, methods);

                self.env.define(&name, Literal::CallLit(class));
                Ok(None)
            }
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
            },
            Call(f, args, line) => {
                let mut callee = self.evaluate_expr(*f)?;
                if let CallLit(ref mut c) = callee {
                    if args.len() != c.arity() {
                        return Err(format!("Expected {} arguments but got {}.", c.arity(), args.len()));
                    }
                    let mut evaled_args = Vec::<Literal>::new();
                    for arg in args.iter() {
                        evaled_args.push(self.evaluate_expr(*arg.clone())?); // TODO: get rid of clone statement if possible
                    }
                    self.call(c, evaled_args)
                } else {
                    Err(String::from("Can only call functions and classes."))
                }
            },
            Getter(obj, name) => {
                let src = self.evaluate_expr(*obj);
                let cpy = src.clone();
                match src {
                    Ok(Literal::InstLit(inst)) => {
                        let out = inst.get(&name)?;
                        if let CallLit(Callable::Function(_, _, _, Some(ref c), _)) = out {
                            let Literal::InstLit(this) = cpy.unwrap() else { panic!("Copy of an instance was somehow not an instance") };
                            if let Callable::Class(_, Some(sc), _) = this.get_class() {
                                c.borrow_mut().define("super", Literal::CallLit(*sc.clone()));  
                            }
                            c.borrow_mut().define("this", Literal::InstLit(this));  
                        }
                        Ok(out)
                    },
                    Ok(_) => Err(String::from("Only instances have properties.")),
                    Err(e) => Err(e),
                }
            }
            Setter(obj, name, value) => {
                match self.evaluate_expr(*obj) {
                    Ok(Literal::InstLit(ref mut inst)) => {
                        let resolved_value = self.evaluate_expr(*value);
                        inst.set(&name, resolved_value?);
                        Ok(inst.get(&name)?)
                    },
                    Ok(_) => Err(String::from("Only instances have fields.")),
                    Err(e) => Err(e),
                }
            }
            This => {
                Ok(self.env.get("this")?)
            }
            Super(method) => {
                let super_class = self.env.get("super")?;
                if let Literal::CallLit(sc) = super_class {
                    Ok(Literal::CallLit(sc.find_method(&method)?))
                } else {
                    Err(String::from("Call to superclass somehow returned non-method."))
                }
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

    fn call(&mut self, callee: &mut Callable, args: Vec<Literal>) -> Result<Literal, String> {
        match callee {
            Callable::Function(name, arg_names, body, ref mut closure, is_init) => {
                self.env.mount_closure(closure);
                self.env.lower_scope();

                let mut name_iter = arg_names.iter().peekable();
                let mut arg_iter = args.iter().peekable();
                while name_iter.peek() != None && arg_iter.peek() != None {
                    let var = name_iter.next().unwrap_or_else(|| panic!("Impossible unwrap fail"));
                    let arg = arg_iter.next().unwrap_or_else(|| panic!("Impossible unwrap fail")).clone();
                    self.env.define(var, arg);
                }

                let result = self.evaluate_stmt(Statement::Block(body.clone()));
                let mut output = match result {
                    Ok(None) => Ok(Literal::Nil),
                    Ok(Some(lit)) => Ok(lit),
                    Err(e) => Err(e),
                };
                if *is_init {
                    output = match output {
                        Ok(_) => self.env.get("this"),
                        Err(e) => Err(e),
                    };
                };

                self.env.raise_scope().expect("Call execution structure should guarantee valid scope raise");
                self.env.unmount_closure().expect("Call execution structure should guarantee valid unmount");
                
                output
                
            },
            Callable::Class(name, super_class, methods) => {
                let inst = Instance::new(Callable::Class(name.clone(), super_class.clone(), methods.clone()));
                if inst.has_initializer() {
                    let mut init = self.evaluate_expr(Getter(Box::new(LitExp(InstLit(inst.clone()))), String::from("init")));
                    if let Ok(CallLit(ref mut c)) = init { self.call(c, args)?; }
                    else { panic!("Initializer failed to resolve to callable.") }
                }
                Ok(Literal::InstLit(inst))
            }
            Callable::Clock => {
                let now = SystemTime::now();
                let time_ms = now.duration_since(UNIX_EPOCH).expect("Got time before unix epoch").as_millis() as f64;
                Ok(Literal::Number(time_ms/1000.0))
            },
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

    mod expressions {
        use super::*;

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
    }

    mod control_flow {
        use super::*;

        mod conditionals {
            use super::*;
    
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
    
        mod loops {
            use super::*;
    
            #[test]
            fn test_while() {
                let mut intp = LoxInterpreter::new();
                let program = string_to_program(
                    "var i = 0;\nwhile (i < 5) print i = i + 1;"
                );
                let output = intp.interpret(program).expect("Error while interpreting program");
            
                let expected = "1\n2\n3\n4\n5";
    
                assert_eq!(expected, output, "Expected left output; recieved right");
            }
    
            #[test]
            fn test_for() {
                let mut intp = LoxInterpreter::new();
                let program = string_to_program(
                    "for (var i = 1; i <= 64; i = i * 2) print i;"
                );
                dbg!(&program);
                let output = intp.interpret(program).expect("Error while interpreting program");
            
                let expected = "1\n2\n4\n8\n16\n32\n64";
    
                assert_eq!(expected, output, "Expected left output; recieved right");
            }
    
        }

    }
    
    mod functions {
        use super::*;

        #[test]
        fn test_clock() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(
                // This isn't a perfect test, but at least confirms that
                // clock() is returning a non-constant, increasing value.
                // The for loops are just to stall for extra time.
                "var a = clock();\nfor (var i = 1; i <= 100; i = i + 1) {for (var j = 1; j <= 100; j = j + 1) {}}\nvar b = clock();\nprint(a == b);\nprint(a<b);"
            );
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "false\ntrue";

            assert_eq!(expected, output, "clock() does not appear to be outputting strictly increasing values");
        }

        #[test]
        fn test_manyvars_func() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "fun sum(a, b, c, d) {\n",
                "  return a + b + c + d;\n",
                "}\n",
                "\n",
                "print sum(1, 2, 3, 4);\n",
                "print sum(4, 5, 6, 7);\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "10\n22";

            assert_eq!(expected, output, "4-input sum function provided unexpected output");
        }

        #[test]
        fn test_curried_func() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "fun sum(a, b, c) {\n",
                "  fun sum_inner() {\n",
                "    return a + b + c;\n",
                "  }\n",
                "  return sum_inner;\n",
                "}\n",
                "\n",
                "print sum(1, 2, 3)();\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "6";

            assert_eq!(expected, output, "Curried function provided unexpected output");
        }

        #[test]
        fn test_recursive_func() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "fun fib(n) {\n",
                "  if (n <= 1) return n;\n",
                "  return fib(n - 2) + fib(n - 1);\n",
                "}\n",
                "\n",
                "for (var i = 0; i < 10; i = i + 1) {\n",
                "  print fib(i);\n",
                "}\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "0\n1\n1\n2\n3\n5\n8\n13\n21\n34";

            assert_eq!(expected, output, "Recursive fibonacci function provided unexpected output");
        }

        #[test]
        fn test_closure_func() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "fun makeCounter() {\n",
                "    var i = 0;\n",
                "    fun count() {\n",
                "        i = i + 1;\n",
                "        print i;\n",
                "    }\n",
                "    \n",
                "    return count;\n",
                "}\n",
                "\n",
                "var counter = makeCounter();\n",
                "for (var i = 0; i < 5; i = i + 1) {\n",
                "    counter();\n",
                "}",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "1\n2\n3\n4\n5";

            assert_eq!(expected, output, "Closure-based counter provided unexpected output");
        }

        #[test]
        fn test_static_scoping() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "var a = \"global\";\n",
                "{\n",
                "  fun showA() {\n",
                "    print a;\n",
                "  }\n",
                "\n",
                "  showA();\n",
                "  var a = \"block\";\n",
                "  showA();\n",
                "}\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "global\nglobal";

            assert_eq!(expected, output, "Closure-based counter provided unexpected output");
        }
    }

    mod classes {
        use super::*;

        #[test]
        fn test_empty_class() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Bagel {}",
                "var bagel = Bagel();",
                "print bagel;",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "<Bagel instance>";

            assert_eq!(expected, output, "Empty class provided unexpected output");
        }

        #[test]
        fn test_basic_fields() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Bacon {}\n",
                "var bacon = Bacon();\n",
                "bacon.tasty = \"Yep!\";\n",
                "print bacon.tasty;",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "Yep!";

            assert_eq!(expected, output, "Basic field get provided unexpected output");
        }

        #[test]
        fn test_basic_method() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Bacon {\n",
                "    eat() {\n",
                "        print \"Crunch crunch crunch!\";\n",
                "    }\n",
                "}\n",
                "Bacon().eat(); // Prints \"Crunch crunch crunch!\"."
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "Crunch crunch crunch!";

            assert_eq!(expected, output, "Basic method provided unexpected output");
        }

        #[test]
        fn test_this_access_method() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Cake {\n",
                "    taste() {\n",
                "        var adjective = \"delicious\";\n",
                "        print \"The \" + this.flavor + \" cake is \" + adjective + \"!\";\n",
                "    }\n",
                "}\n",
                "\n",
                "var cake = Cake();\n",
                "cake.flavor = \"German chocolate\";\n",
                "cake.taste();",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "The German chocolate cake is delicious!";

            assert_eq!(expected, output, "Method relying on 'this' provided unexpected output");
        }

        #[test]
        fn test_initializer() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Foo {\n",
                "    init(one, two) {\n",
                "        this.foo = one;\n",
                "        this.bar = two;\n",
                "        print one + two;\n",
                "    }\n",
                "}\n",
                "\n",
                "var foo = Foo(\"foo\", \"bar\");\n",
                "print foo.foo;\n",
                "print foo.bar;\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "foobar\nfoo\nbar";

            assert_eq!(expected, output, "Running class initializer resulted in unexpected output");
        }

        #[test]
        fn test_initializer_recall() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Foo {\n",
                "    init() {\n",
                "        print this;\n",
                "        return;\n",
                "        print \"early return didn't work\";\n",
                "    }\n",
                "}\n",
                "\n",
                "var foo = Foo();\n",
                "print foo.init();\n",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "<Foo instance>\n<Foo instance>\n<Foo instance>";

            assert_eq!(expected, output, "Running initializer a second time provided unexpected output");
        }

        #[test]
        fn test_simple_inheritance() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class Doughnut {\n",
                "    cook() {\n",
                "        print \"Fry until golden brown.\";\n",
                "    }\n",
                "}\n",
                "\n",
                "class BostonCream < Doughnut {\n",
                "    cook() {\n",
                "        super.cook();\n",
                "        print \"Pipe full of custard and coat with chocolate.\";\n",
                "    }\n",
                "}\n",
                "\n",
                "BostonCream().cook();",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "Fry until golden brown.\nPipe full of custard and coat with chocolate.";

            assert_eq!(expected, output, "Running simple inherited method provided unexpected output");
        }

        #[test]
        fn test_chained_inheritance() {
            let mut intp = LoxInterpreter::new();
            let program = string_to_program(concat!(
                "class A {\n",
                "  method() {\n",
                "    print \"A method\";\n",
                "  }\n",
                "}\n",
                "\n",
                "class B < A {\n",
                "  method() {\n",
                "    print \"B method\";\n",
                "  }\n",
                "\n",
                "  test() {\n",
                "    super.method();\n",
                "  }\n",
                "}\n",
                "\n",
                "class C < B {}\n",
                "\n",
                "C().test();",
            ));
            let output = intp.interpret(program).expect("Error while interpreting program");
        
            let expected = "B method";

            assert_eq!(expected, output, "Running method from inheritance chain provided unexpected output");
        }

    }

}