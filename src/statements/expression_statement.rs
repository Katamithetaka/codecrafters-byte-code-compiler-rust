use crate::{
    compiler::CodeGenerator,
    expressions::{Expression, Expressions},
    statements::Statement,
};

#[derive(Debug)]
pub struct ExprStatement<'a> {
    expr: Expressions<'a>,
}
impl<'a> ExprStatement<'a> {
    pub fn new(expr: Expressions<'a>) -> Self {
        Self { expr }
    }
}
impl Statement for ExprStatement<'_> {}

impl CodeGenerator for ExprStatement<'_> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = match dst_register {
            Some(a) => a,
            None => reserved_registers.iter().max().copied().unwrap_or(0),
        };

        self.expr
            .write_expression(chunk, Some(dist), reserved_registers)?;

        Ok(())
    }
}
