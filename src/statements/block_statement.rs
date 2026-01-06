use crate::{
    compiler::CodeGenerator,
    statements::{Statement, Statements},
};

#[derive(Debug)]
pub struct BlockStatement<'a> {
    pub statements: Vec<Statements<'a>>,
    pub begin_line: usize,
    pub end_line: usize,
}

impl BlockStatement<'_> {
    pub fn new<'a>(
        statements: Vec<Statements<'a>>,
        begin_line: usize,
        end_line: usize,
    ) -> BlockStatement<'a> {
        BlockStatement {
            statements,
            begin_line,
            end_line,
        }
    }
}

impl<'a> Statement<'a> for BlockStatement<'a> {}

impl<'a> CodeGenerator<'a> for BlockStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        chunk.write_stack_push(self.begin_line as i32);
        for i in &mut self.statements {
            i.write_expression(chunk, dst_register, reserved_registers.clone())?;
        }
        chunk.write_stack_pop(self.end_line as i32);

        Ok(())
    }
}
