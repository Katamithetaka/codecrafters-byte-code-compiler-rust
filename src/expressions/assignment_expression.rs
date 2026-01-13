use std::fmt::Display;

use crate::{
    compiler::CodeGenerator,
    expressions::{
        Expression, Expressions, Value,
        identifier::{Identifier, IdentifierKind},
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
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);
        self.rhs
            .write_expression(chunk, Some(dist), reserved_registers)?;
        match self.lhs.kind {
            IdentifierKind::GlobalScope => {
                let constant = chunk
                    .get_or_write_constant(Value::String(self.lhs.token), self.lhs.line as i32);

                chunk.write_set_global(constant, dist, self.lhs.line as i32);
            }
            IdentifierKind::LocalScope { slot} => {
                chunk.write_set_local(dist, slot, self.lhs.line as i32);
            }
            IdentifierKind::UpperScope { .. } => {
                todo!();
                // chunk.write_set_local(dist, slot, self.lhs.line as i32);
            }
        }
        Ok(())
    }
}
