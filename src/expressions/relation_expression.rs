use std::fmt::Display;

use crate::{
    compiler::{CodeGenerator, chunk::Chunk, instructions::Instructions},
    expressions::{Expression, Expressions},
};

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
    pub lhs: Box<Expressions<'a>>,
    pub rhs: Box<Expressions<'a>>,
    pub op: RelationalOp,
    line_number: usize,
}

impl<'a> Display for RelationalExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl<'a> RelationalExpression<'a> {
    pub fn new(op: RelationalOp, lhs: Box<Expressions<'a>>, rhs: Box<Expressions<'a>>) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
            op,
        };
    }
}

impl<'a> Expression<'a> for RelationalExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<'a> CodeGenerator<'a> for RelationalExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let instruction = match self.op {
            RelationalOp::Greater => Instructions::Gt,
            RelationalOp::GreaterEqual => Instructions::GtEq,
            RelationalOp::Less => Instructions::Lt,
            RelationalOp::LessEqual => Instructions::LtEq,
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
