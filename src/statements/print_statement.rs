use crate::{compiler::CodeGenerator, expressions::Expression, statements::Statement};

#[derive(Debug)]
pub struct PrintStatement<'a> {
    expr: Box<dyn Expression + 'a>
}
impl<'a> PrintStatement<'a> {
    pub fn new(expr: Box<dyn Expression + 'a>) -> Self {
        Self {
            expr
        }
    }
}
impl Statement for PrintStatement<'_>{}

impl CodeGenerator for PrintStatement<'_> {
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
        
        self.expr.write_expression(chunk, Some(dist), reserved_registers);
        chunk.write_print(dist, self.expr.line_number() as i32);
        
        
        
        Ok(())
    }
}
