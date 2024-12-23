use crate::components as lox;
use lox::instructions::node::*;
use Literal::*;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Expression {
    LitExp(Literal),
    Unary(UnaryOp, Box<Expression>),
    Binary { left: Box<Expression>, op: BinaryOp, right: Box<Expression> },
    Logical { left: Box<Expression>, op: LogicOp, right: Box<Expression> },
    Identifier(String),
    Grouping(Box<Expression>),
    Assignment(String, Box<Expression>),
    Call(Box<Expression>, Vec<Box<Expression>>, usize),
    Getter(Box<Expression>, String),
    Setter(Box<Expression>, String, Box<Expression>), // Object, name, value
    This,
    Super(String),
    // usize is used for line numbers in error reporting
    // TODO: maybe should shift the program to use this more broadly?
}
use Expression::*;

impl Expression {
    pub fn boxed_literal(l: Literal) -> Box<Expression> {
        Box::new(LitExp(l))
    }

    pub fn boxed_number(n: f64) -> Box<Expression> {
        Box::new(LitExp(Number(n)))
    }
    pub fn boxed_string(s: &str) -> Box<Expression> {
        Box::new(LitExp(StringData(String::from(s))))
    }
    pub fn boxed_boolean(b: bool) -> Box<Expression> {
        Box::new(LitExp(Boolean(b)))
    }
    pub fn boxed_nil() -> Box<Expression> {
        Box::new(LitExp(Nil))
    }

    pub fn boxed_unary(op: UnaryOp, e: Box<Expression>) -> Box<Expression> {
        Box::new(Unary(op, e))
    }
    pub fn boxed_binary(
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>
    ) -> Box<Expression> {
        Box::new(Binary{ left, op, right })
    }
    pub fn boxed_logical(
        left: Box<Expression>,
        op: LogicOp,
        right: Box<Expression>
    ) -> Box<Expression> {
        Box::new(Logical{ left, op, right })
    }

    pub fn boxed_identifier(s: &str) -> Box<Expression> {
        Box::new(Identifier(String::from(s)))
    }
    pub fn boxed_grouping(e: Box<Expression>) -> Box<Expression> {
        Box::new(Grouping(e))
    }
    pub fn boxed_assignment(s: &str, e: Box<Expression>) -> Box<Expression> {
        Box::new(Assignment(String::from(s), e))
    }
    pub fn boxed_call(f: Box<Expression>, args: Vec<Box<Expression>>, line: usize) -> Box<Expression> {
        Box::new(Call(f, args, line))
    }
    pub fn boxed_getter(obj: Box<Expression>, name: &str) -> Box<Expression> {
        Box::new(Getter(obj, String::from(name)))
    }
    pub fn boxed_setter(obj: Box<Expression>, name: &str, value: Box<Expression>) -> Box<Expression> {
        Box::new(Setter(obj, String::from(name), value))
    }
    
    pub fn boxed_this() -> Box<Expression> {
        Box::new(This)
    }
    pub fn boxed_super(name: &str) -> Box<Expression> {
        Box::new(Super(String::from(name)))
    }
}

