use std::fmt::Display;

use crate::{
    Token,
    compiler::{CodeGenerator, chunk::Chunk},
    expressions::{Expression, Value, expect_ok},
    scanner::{Keyword, TokenKind, TokenValue},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifierKind {
    GlobalScope,
    LocalScope,
}

#[derive(Debug)]
pub struct Identifier<'a> {
    pub token: &'a str,
    pub line: usize,
    pub kind: IdentifierKind,
}

impl<'a> Display for Identifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl<'a> Identifier<'a> {
    pub fn new(token: &'a Token<'a>) -> Self {
        return Self {
            token: token.lexeme,
            line: token.line,
            kind: IdentifierKind::GlobalScope,
        };
    }
}

impl<'a> Expression for Identifier<'a> {
    fn line_number(&self) -> usize {
        self.line
    }
}

impl<'a> CodeGenerator for Identifier<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        match self.kind {
            IdentifierKind::GlobalScope => {
                let ident_name = Value::String(self.token.to_string());
                let constant = chunk.get_or_write_constant(ident_name, self.line as i32);
                let dst = self.dst_or_default(dst_register, &reserved_registers);

                chunk.write_get_global(constant, dst, self.line as i32);
            }
            IdentifierKind::LocalScope => unimplemented!(),
        }

        Ok(())
    }
}
