use std::collections::HashMap;

use crate::{cursor::Cursor, lexer::Token};

enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Equal,
    NotEqual,
    Lesser,
    LesserEqual,
    Greater,
    GreaterEqual,

    And,
    Or,
}

enum UnaryOperator {
    Invert,
    Negative,
    Positive
}

enum RangeType {
    Exclusive,
    Inclusive
}

enum Expression {
    Int(i32),
    Float(f64),
    Boolean(bool),
    String(String),
    Identifier(String),

    Array(Vec<Expression>),
    Object(HashMap<String, Expression>),

    BinaryOperation {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: BinaryOperator
    },
    UnaryOperator {
        expression: Box<Expression>,
        operator: UnaryOperator 
    },
    FunctionCall {
        function: Box<Expression>,
        args: Vec<Expression>
    },
    ArrayIndex {
        target: Box<Expression>,
        index: Box<Expression>
    },
    Field {
        target: Box<Expression>,
        name: String
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        range_type: RangeType
    }
}

pub struct Parser<'a> {
    tokens: Cursor<'a, Token> 
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: Cursor::new(tokens)
        }
    }
}
