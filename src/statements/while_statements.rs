use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler},
    expressions::{EvaluateError, EvaluateErrorDetails, Expression, Expressions},
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
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
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

            chunk.write_jump_if_false_placeholder(dst, self.expression.line_number() as i32)
        };

        self.statement
            .write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;
        let mut chunk = chunk.borrow_mut();

        chunk.write_goto(loop_offset as u16, self.expression.line_number() as i32);

        match chunk.update_jump(offset) {
            Ok(_) => {}
            Err(_) => {
                return Err(EvaluateError {
                    error: EvaluateErrorDetails::CodeTooLong,
                    line: self.expression.line_number(),
                });
            }
        }

        Ok(())
    }
}
