use std::fmt::Display;

use crate::expressions::{Expression, expect_ok};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinaryOp {
    Plus,
    Minus,
    Star,
    Slash,
}

impl<'a> Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Plus => f.write_str("+"),
            BinaryOp::Minus => f.write_str("-"),
            BinaryOp::Star => f.write_str("*"),
            BinaryOp::Slash => f.write_str("/"),
        }
    }
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub lhs: Box<dyn Expression>,
    pub rhs: Box<dyn Expression>,
    pub op: BinaryOp,
    line_number: usize,
}

impl Display for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl BinaryExpression {
    pub fn new(op: BinaryOp, lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
            op,
        };
    }
}

impl Expression for BinaryExpression {
    fn line_number(&self) -> usize {
        self.line_number
    }

    fn evaluate(&mut self) -> super::Result {
        let left = self.lhs.evaluate();
        let left = match expect_ok(left) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(v)) => v,
        };

        self.line_number = self.rhs.line_number();
        let right = self.rhs.evaluate();
        let right = match expect_ok(right) {
            Err(v) => return Err(v),
            Ok(None) => return self.err(super::EvaluateErrorDetails::ExpectedValue),
            Ok(Some(v)) => v,
        };

        self.line_number = self.lhs.line_number();

        match left.binary_op_compatible(&right, self.op) {
            Some(err) => return self.err(err),
            None => (),
        };

        self.ok(Some(match self.op {
            BinaryOp::Plus => left.add(&right),
            BinaryOp::Minus => left.sub(&right),
            BinaryOp::Star => left.mult(&right),
            BinaryOp::Slash => left.div(&right),
        }))
    }
}
