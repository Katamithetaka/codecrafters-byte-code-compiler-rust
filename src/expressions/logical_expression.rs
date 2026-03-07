use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler, instructions::Instructions, int_types::{line_type, register_index_type}},
    expressions::{Expression, Expressions},
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
    line_number: line_type,
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
    fn line_number(&self) -> line_type{
        self.line_number
    }
}

impl<'a> CodeGenerator<'a> for LogicalExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        let boolean_dst = match self.op {
            LogicalOp::Or => {
                self.lhs
                    .write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;

                chunk.borrow_mut().write_unary(
                    Instructions::Bang,
                    dst,
                    self.next_dst(dst, 1, &reserved_registers),
                    self.lhs.line_number(),
                );
                self.next_dst(dst, 1, &reserved_registers)
            }
            LogicalOp::And => {
                self.lhs
                    .write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;

                dst
            }
        };


        let offset =
            chunk.borrow_mut().write_jump_if_false_placeholder(boolean_dst, self.lhs.line_number())?;

        self.rhs
            .write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;

        chunk.borrow_mut().update_jump(offset)?;

        Ok(())
    }
}
