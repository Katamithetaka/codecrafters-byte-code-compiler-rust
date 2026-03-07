use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler, int_types::{instruction_length_type, register_index_type}},
    expressions::{Expression, Expressions},
    statements::{Statement, Statements},
};

#[derive(Debug)]
pub struct WhileStatement<'a> {
    pub expression: Expressions<'a>,
    pub statement: Box<Statements<'a>>,
}

impl WhileStatement<'_> {
    pub fn new<'a>(
        expression: Expressions<'a>,
        statement: Box<Statements<'a>>,
    ) -> WhileStatement<'a> {
        WhileStatement {
            expression,
            statement,
        }
    }
}

impl<'a> Statement<'a> for WhileStatement<'a> {}

impl<'a> CodeGenerator<'a> for WhileStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        let loop_offset = {
            let chunk = chunk.borrow();
            chunk.chunk.code.len()
        };
        self.expression
            .write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;

        let offset = {
            let mut chunk = chunk.borrow_mut();

            chunk.write_jump_if_false_placeholder(dst, self.expression.line_number())?
        };

        self.statement
            .write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;
        let mut chunk = chunk.borrow_mut();

        chunk.write_goto(loop_offset as instruction_length_type, self.expression.line_number());

        chunk.update_jump(offset)?;

        Ok(())
    }
}
