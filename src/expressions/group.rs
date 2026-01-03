use std::fmt::Display;

use crate::{
    Token,
    expressions::{Expression, Value, expect_ok},
    scanner::{Keyword, TokenKind, TokenValue},
};

#[derive(Debug)]
pub struct Group<'a> {
    expr: Box<dyn Expression + 'a>,
}

impl<'a> Display for Group<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(group {})", self.expr)
    }
}

impl<'a> Group<'a> {
    pub fn new(expr: Box<dyn Expression + 'a>) -> Self {
        return Self { expr };
    }
}

impl<'a> Expression for Group<'a> {
    fn line_number(&self) -> usize {
        self.expr.line_number()
    }

    fn evaluate(&mut self) -> super::Result {
        return self.expr.evaluate();
    }
}
