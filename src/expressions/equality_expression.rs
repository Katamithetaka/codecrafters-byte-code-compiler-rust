use std::fmt::Display;

use crate::expressions::{Expression, expect_ok};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EqualityOp {
    EqualEqual,
    BangEqual,
}

impl<'a> Display for EqualityOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EqualityOp::EqualEqual => f.write_str("=="),
            EqualityOp::BangEqual => f.write_str("!="),
        }
    }
}

#[derive(Debug)]
pub struct EqualityExpression<'a> {
    pub lhs: Box<dyn Expression + 'a>,
    pub rhs: Box<dyn Expression + 'a>,
    pub op: EqualityOp,
    line_number: usize,
}

impl<'a> Display for EqualityExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl<'a> EqualityExpression<'a> {
    pub fn new(
        op: EqualityOp,
        lhs: Box<dyn Expression + 'a>,
        rhs: Box<dyn Expression + 'a>,
    ) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
            op,
        };
    }
}

impl<'a> Expression for EqualityExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }

    fn evaluate(&mut self) -> super::Result {
        let left = self.lhs.evaluate();
        let left = match expect_ok(left) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(c)) => c,
        };

        self.line_number = self.rhs.line_number();
        let right = self.rhs.evaluate();
        let right = match expect_ok(right) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(c)) => c,
        };

        self.line_number = self.lhs.line_number();

        self.ok(Some(super::Value::Boolean(match self.op {
            EqualityOp::EqualEqual => left == right,
            EqualityOp::BangEqual => left != right,
        })))
    }
}
