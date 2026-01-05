use std::fmt::Display;

use crate::{
    compiler::{CodeGenerator, instructions::Instructions},
    expressions::{Expression, Expressions, expect_ok},
};

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
pub struct BinaryExpression<'a> {
    pub lhs: Box<Expressions<'a>>,
    pub rhs: Box<Expressions<'a>>,
    pub op: BinaryOp,
    line_number: usize,
}

impl<'a> Display for BinaryExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl<'a> BinaryExpression<'a> {
    pub fn new(op: BinaryOp, lhs: Box<Expressions<'a>>, rhs: Box<Expressions<'a>>) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
            op,
        };
    }
}

impl<'a> Expression for BinaryExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<'a> CodeGenerator for BinaryExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk,
        dst_register: Option<u8>,
        mut reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let instruction = match self.op {
            BinaryOp::Plus => Instructions::Add,
            BinaryOp::Minus => Instructions::Sub,
            BinaryOp::Star => Instructions::Mul,
            BinaryOp::Slash => Instructions::Div,
        };

        crate::compiler::macros::binary_op!(
            instruction,
            dst_register,
            reserved_registers,
            chunk,
            self
        )
    }
}
