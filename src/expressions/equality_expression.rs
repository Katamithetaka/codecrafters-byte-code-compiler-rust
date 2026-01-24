use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler, instructions::Instructions},
    expressions::{Expression, Expressions},
};

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
    pub lhs: Box<Expressions<'a>>,
    pub rhs: Box<Expressions<'a>>,
    pub op: EqualityOp,
    line_number: usize,
}

impl<'a> Display for EqualityExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.lhs, self.rhs)
    }
}

impl<'a> EqualityExpression<'a> {
    pub fn new(op: EqualityOp, lhs: Box<Expressions<'a>>, rhs: Box<Expressions<'a>>) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
            op,
        };
    }
}

impl<'a> Expression<'a> for EqualityExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<'a> CodeGenerator<'a> for EqualityExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let instruction = match self.op {
            EqualityOp::EqualEqual => Instructions::Eq,
            EqualityOp::BangEqual => Instructions::Neq,
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
