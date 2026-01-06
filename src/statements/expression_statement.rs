use crate::{compiler::CodeGenerator, expressions::Expressions, statements::Statement};

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
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);

        self.expr
            .write_expression(chunk, Some(dist), reserved_registers)?;

        Ok(())
    }
}
