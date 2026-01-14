use crate::{
    compiler::CodeGenerator,
    expressions::{Expressions, Value},
    statements::Statement,
};

#[derive(Debug)]
pub struct ReturnStatement<'a> {
    pub expr: Option<Expressions<'a>>,
    pub line_number: usize,
}
impl<'a> ReturnStatement<'a> {
    pub fn new(expr: Option<Expressions<'a>>, line_number: usize) -> Self {
        Self { expr, line_number }
    }
}
impl<'a> Statement<'a> for ReturnStatement<'a> {}

impl<'a> CodeGenerator<'a> for ReturnStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        _dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = 0;

        if let Some(expr) = &mut self.expr {
            expr
                .write_expression(chunk, Some(dist), reserved_registers)?;
        }
        else {
            let constant = chunk.get_or_write_constant(Value::Null, self.line_number as i32);
            chunk.write_load(0, constant, self.line_number as i32);
        }
        chunk.write_function_return( self.line_number as i32);

        Ok(())
    }
    

}
