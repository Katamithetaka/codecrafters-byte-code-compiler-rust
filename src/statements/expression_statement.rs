use std::{cell::RefCell, rc::Rc};

use crate::{compiler::{CodeGenerator, compiler::Compiler, int_types::register_index_type}, expressions::Expressions, statements::Statement};

#[derive(Debug)]
pub struct ExprStatement<'a> {
    pub expr: Expressions<'a>,
}
impl<'a> ExprStatement<'a> {
    pub fn new(expr: Expressions<'a>) -> Self {
        Self { expr }
    }
}
impl<'a> Statement<'a> for ExprStatement<'a> {}

impl<'a> CodeGenerator<'a> for ExprStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);

        self.expr
            .write_expression(chunk, Some(dist), reserved_registers)?;

        Ok(())
    }
}
