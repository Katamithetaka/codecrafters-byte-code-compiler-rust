use std::fmt::Display;

use crate::{
    Token,
    compiler::{CodeGenerator, chunk::Chunk},
    expressions::{Expression, Expressions, Value, expect_ok},
    scanner::{Keyword, TokenKind, TokenValue},
};

#[derive(Debug)]
pub struct Group<'a> {
    expr: Box<Expressions<'a>>,
}

impl<'a> Display for Group<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(group {})", self.expr)
    }
}

impl<'a> Group<'a> {
    pub fn new(expr: Box<Expressions<'a>>) -> Self {
        return Self { expr };
    }
}

impl<'a> Expression for Group<'a> {
    fn line_number(&self) -> usize {
        self.expr.line_number()
    }
}

impl<'a> CodeGenerator for Group<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        self.expr
            .write_expression(chunk, dst_register, reserved_registers)
    }
}
