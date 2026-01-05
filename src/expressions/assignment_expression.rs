use std::fmt::Display;

use crate::{
    compiler::{CodeGenerator, instructions::Instructions},
    expressions::{Expression, Expressions, Value, expect_ok, identifier::Identifier},
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

impl<'a> Expression for AssignmentExpression<'a> {
    fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<'a> CodeGenerator for AssignmentExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk,
        dst_register: Option<u8>,
        mut reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = match dst_register {
            Some(a) => a,
            None => reserved_registers.iter().max().copied().unwrap_or(0),
        };

        self.rhs
            .write_expression(chunk, Some(dist), reserved_registers)?;

        let constant =
            chunk.get_or_write_constant(Value::String(self.lhs.to_string()), self.lhs.line as i32);

        chunk.write_set_global(constant, dist, self.lhs.line as i32);

        Ok(())
    }
}
