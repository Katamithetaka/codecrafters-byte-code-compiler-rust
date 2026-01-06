use std::fmt::Display;

use crate::{
    compiler::{CodeGenerator, chunk::Chunk, instructions::Instructions},
    expressions::{EvaluateError, EvaluateErrorDetails, Expression, Expressions},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogicalOp {
    Or,
    And,
}

impl<'a> Display for LogicalOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalOp::Or => f.write_str("or"),
            LogicalOp::And => f.write_str("and"),
        }
    }
}

#[derive(Debug)]
pub struct LogicalExpression<'a> {
    pub lhs: Box<Expressions<'a>>,
    pub rhs: Box<Expressions<'a>>,
    pub op: LogicalOp,
    line_number: usize,
}

impl<'a> Display for LogicalExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl<'a> LogicalExpression<'a> {
    pub fn new(op: LogicalOp, lhs: Box<Expressions<'a>>, rhs: Box<Expressions<'a>>) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
            op,
        };
    }
}

impl<'a> Expression<'a> for LogicalExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<'a> CodeGenerator<'a> for LogicalExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        let boolean_dst = match self.op {
            LogicalOp::Or => {
                self.lhs
                    .write_expression(chunk, Some(dst), reserved_registers.clone())?;
                chunk.write_unary(
                    Instructions::Bang,
                    dst,
                    dst + 1,
                    self.lhs.line_number() as i32,
                );
                dst + 1
            }
            LogicalOp::And => {
                self.lhs
                    .write_expression(chunk, Some(dst), reserved_registers.clone())?;

                dst
            }
        };
        let offset =
            chunk.write_jump_if_false_placeholder(boolean_dst, self.lhs.line_number() as i32);

        self.rhs
            .write_expression(chunk, Some(dst), reserved_registers.clone())?;

        match chunk.update_jump(offset) {
            Ok(_) => Ok(()),
            Err(_) => Err(EvaluateError {
                error: EvaluateErrorDetails::CodeTooLong,
                line: self.rhs.line_number(),
            }),
        }
    }
}
