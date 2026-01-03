use std::fmt::Display;

use crate::expressions::{Expression, Value, expect_ok};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnaryOp {
    Bang,
    Minus,
}

impl<'a> Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bang => f.write_str("!"),
            Self::Minus => f.write_str("-"),
        }
    }
}

#[derive(Debug)]
pub struct UnaryExpression<'a> {
    pub rhs: Box<dyn Expression + 'a>,
    pub op: UnaryOp,
}

impl<'a> Display for UnaryExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.op, self.rhs)
    }
}

impl<'a> UnaryExpression<'a> {
    pub fn new(op: UnaryOp, rhs: Box<dyn Expression + 'a>) -> Self {
        return Self { rhs, op };
    }
}

impl<'a> Expression for UnaryExpression<'a> {
    fn line_number(&self) -> usize {
        self.rhs.line_number()
    }

    fn evaluate(&mut self) -> super::Result {
        let left = self.rhs.evaluate();
        let left = match expect_ok(left) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(v)) => v,
        };

        self.ok(Some(match (self.op, left) {
            (UnaryOp::Bang, left) => Value::Boolean(!left.is_truthy()),
            (UnaryOp::Minus, Value::Number(v)) => Value::Number(-v),
            _ => return self.err(super::EvaluateErrorDetails::UnaryNumberOp),
        }))
    }
}
