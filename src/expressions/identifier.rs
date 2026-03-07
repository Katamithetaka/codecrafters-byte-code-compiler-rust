use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    Token,
    compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}, int_types::{line_type, register_index_type}},
    expressions::Expression,
};



#[derive(Debug)]
pub struct Identifier<'a> {
    pub token: &'a str,
    pub line: line_type,
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
            line: token.line as line_type,
        };
    }
}

impl<'a> Expression<'a> for Identifier<'a> {
    fn line_number(&self) -> line_type{
        self.line
    }
}

impl<'a> CodeGenerator<'a> for Identifier<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        let mut chunk = chunk.borrow_mut();
        match chunk.resolve_variable(self.token)? {
            ResolvedVar::Global(slot) => {
                chunk.write_get_global(slot, dst, self.line as line_type);
            }
            ResolvedVar::Local(slot) => {
                chunk.write_get_local(dst, slot, self.line as line_type);
            }
            ResolvedVar::Upvalue(slot) => {
                chunk.write_get_upvalue(dst, slot, self.line as line_type);
            }
        }

        Ok(())
    }
}
