use std::fmt::Display;

use crate::expressions::{Expression, expect_ok};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RelationalOp {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl<'a> Display for RelationalOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationalOp::Less => f.write_str("<"),
            RelationalOp::LessEqual => f.write_str("<="),
            RelationalOp::Greater => f.write_str(">"),
            RelationalOp::GreaterEqual => f.write_str(">="),
        }
    }
}

#[derive(Debug)]
pub struct RelationalExpression<'a> {
    pub lhs: Box<dyn Expression + 'a>,
    pub rhs: Box<dyn Expression + 'a>,
    pub op: RelationalOp,
    line_number: usize,
}

impl<'a> Display for RelationalExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl<'a> RelationalExpression<'a> {
    pub fn new(
        op: RelationalOp,
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

impl<'a> Expression for RelationalExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }

    fn evaluate(&mut self) -> super::Result {
        let left = self.lhs.evaluate();
        let left = match expect_ok(left) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(super::Value::Number(v))) => v,
            Ok(Some(_)) => return self.err(super::EvaluateErrorDetails::BinaryNumberOp),
        };

        self.line_number = self.rhs.line_number();
        let right = self.rhs.evaluate();
        let right = match expect_ok(right) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(super::Value::Number(v))) => v,
            Ok(Some(_)) => return self.err(super::EvaluateErrorDetails::BinaryNumberOp),
        };

        self.line_number = self.lhs.line_number();

        self.ok(Some(super::Value::Boolean(match self.op {
            RelationalOp::Less => left < right,
            RelationalOp::Greater => left > right,
            RelationalOp::GreaterEqual => left >= right,
            RelationalOp::LessEqual => left <= right,
        })))
    }
}
