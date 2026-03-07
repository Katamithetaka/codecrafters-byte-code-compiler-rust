use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler, instructions::Instructions, int_types::{line_type, register_index_type}},
    expressions::{Expression, Expressions},
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
    line_number: line_type,
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

impl<'a> Expression<'a> for BinaryExpression<'a> {
    fn line_number(&self) -> line_type {
        self.line_number
    }
}

impl<'a> CodeGenerator<'a> for BinaryExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
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
