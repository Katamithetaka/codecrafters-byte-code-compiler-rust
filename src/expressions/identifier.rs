use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    Token,
    compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}},
    expressions::{Expression},
};



#[derive(Debug)]
pub struct Identifier<'a> {
    pub token: &'a str,
    pub line: usize,
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
        };
    }
}

impl<'a> Expression<'a> for Identifier<'a> {
    fn line_number(&self) -> usize {
        self.line
    }
}

impl<'a> CodeGenerator<'a> for Identifier<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        let mut chunk = chunk.borrow_mut();
        match chunk.resolve_variable(self.token)? {
            ResolvedVar::Global(slot) => {
                chunk.write_get_global(slot, dst, self.line as i32);
            }
            ResolvedVar::Local(slot) => {
                chunk.write_get_local(dst, slot, self.line as i32);
            }
            ResolvedVar::Upvalue(slot) => {
                chunk.write_get_upvalue(dst, slot, self.line as i32);
            }
        }

        Ok(())
    }
}
