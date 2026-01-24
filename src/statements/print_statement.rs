use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler},
    expressions::{Expression, Expressions},
    statements::Statement,
};

#[derive(Debug)]
pub struct PrintStatement<'a> {
    pub expr: Expressions<'a>,
}
impl<'a> PrintStatement<'a> {
    pub fn new(expr: Expressions<'a>) -> Self {
        Self { expr }
    }
}
impl<'a> Statement<'a> for PrintStatement<'a> {}

impl<'a> CodeGenerator<'a> for PrintStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);

        self.expr
            .write_expression(chunk.clone(), Some(dist), reserved_registers)?;
        let mut chunk = chunk.borrow_mut();
        chunk.write_print(dist, self.expr.line_number() as i32);

        Ok(())
    }
}
