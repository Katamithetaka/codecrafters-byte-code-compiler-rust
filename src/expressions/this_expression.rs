use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    Token, compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}, int_types::{line_type, register_index_type}}, expressions::Expression, prelude::EvaluateError, value::{EvaluateErrorDetails, callable::FunctionKind}
};



#[derive(Debug)]
pub struct This<'a> {
    pub token: &'a str,
    pub line: line_type,
}

impl<'a> Display for This<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl<'a> This<'a> {
    pub fn new(token: &'a Token<'a>) -> Self {
        return Self {
            token: token.lexeme,
            line: token.line as line_type,
        };
    }
}

impl<'a> Expression<'a> for This<'a> {
    fn line_number(&self) -> line_type{
        self.line
    }
}

impl<'a> CodeGenerator<'a> for This<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);

        if !chunk.borrow().is_in_method() {
            return Err(EvaluateError {
                error: EvaluateErrorDetails::ParserError(crate::ParserError { error: crate::ast_parser::ParserErrorDetails::InvalidThisUsage, line: self.line_number() }),
                line: self.line_number()
            })
        }

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
