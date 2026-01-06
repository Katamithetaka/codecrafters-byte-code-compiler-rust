use std::fmt::Display;

use crate::{
    compiler::{CodeGenerator, chunk::Chunk, instructions::Instructions},
    expressions::{Expression, Expressions},
};

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
    pub rhs: Box<Expressions<'a>>,
    pub op: UnaryOp,
}

impl<'a> Display for UnaryExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.op, self.rhs)
    }
}

impl<'a> UnaryExpression<'a> {
    pub fn new(op: UnaryOp, rhs: Box<Expressions<'a>>) -> Self {
        return Self { rhs, op };
    }
}

impl<'a> Expression<'a> for UnaryExpression<'a> {
    fn line_number(&self) -> usize {
        self.rhs.line_number()
    }
}

impl<'a> CodeGenerator<'a> for UnaryExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let my_dst_register = match dst_register {
            Some(v) => v,
            None => reserved_registers.iter().max().copied().unwrap_or(0) + 1,
        };

        self.rhs
            .write_expression(chunk, Some(my_dst_register), reserved_registers)?;

        let dst = match dst_register {
            Some(dst) => dst,
            None => my_dst_register,
        };

        match self.op {
            UnaryOp::Bang => chunk.write_unary(
                Instructions::Bang,
                my_dst_register,
                dst,
                self.line_number() as i32,
            ),
            UnaryOp::Minus => chunk.write_unary(
                Instructions::Negate,
                my_dst_register,
                dst,
                self.line_number() as i32,
            ),
        };

        Ok(())
    }
}
