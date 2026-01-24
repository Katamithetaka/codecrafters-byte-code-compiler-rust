use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}},
    expressions::{
        Expression, Expressions,
        identifier::Identifier,
    },
};

#[derive(Debug)]
pub struct AssignmentExpression<'a> {
    pub lhs: Identifier<'a>,
    pub rhs: Box<Expressions<'a>>,
    line_number: usize,
}

impl<'a> Display for AssignmentExpression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(= {} {})", self.lhs, self.rhs)
    }
}

impl<'a> AssignmentExpression<'a> {
    pub fn new(lhs: Identifier<'a>, rhs: Box<Expressions<'a>>) -> Self {
        return Self {
            line_number: lhs.line_number(),
            lhs,
            rhs,
        };
    }
}

impl<'a> Expression<'a> for AssignmentExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<'a> CodeGenerator<'a> for AssignmentExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);
        self.rhs
            .write_expression(chunk.clone(), Some(dist), reserved_registers)?;
        let mut chunk = chunk.borrow_mut();
        match chunk.resolve_variable(self.lhs.token)? {
            ResolvedVar::Local(i) => {
                chunk.write_set_local(dist, i, self.line_number as i32);
            },
            ResolvedVar::Upvalue(slot) => {
                chunk.write_set_upvalue(dist, slot, self.line_number as i32);
            },
            ResolvedVar::Global(varint) => {
                chunk.write_set_global(varint, dist, self.line_number as i32);
            },
        }
        Ok(())
    }
}
