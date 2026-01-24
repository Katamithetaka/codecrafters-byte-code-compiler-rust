use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler},
    expressions::{EvaluateError, EvaluateErrorDetails, Expressions, Value},
    statements::{Statement, Statements},
};

#[derive(Debug)]
pub struct IfStatement<'a> {
    pub statements: Vec<(Option<Expressions<'a>>, Statements<'a>, usize)>,
}

impl IfStatement<'_> {
    pub fn new<'a>(
        statements: Vec<(Option<Expressions<'a>>, Statements<'a>, usize)>,
    ) -> IfStatement<'a> {
        IfStatement { statements }
    }
}

impl<'a> Statement<'a> for IfStatement<'a> {}

impl<'a> CodeGenerator<'a> for IfStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        let mut jmps = vec![];

        for (expr, statement, line) in self.statements.iter_mut() {
            match expr {
                Some(expr) => {
                    expr.write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?
                }
                None => {
                    let mut chunk = chunk.borrow_mut();
                    let constant = chunk.get_or_write_constant(Value::Boolean(true), *line as i32);
                    chunk.write_load(dst, constant, *line as i32);
                }
            };


            let offset = chunk.borrow_mut().write_jump_if_false_placeholder(dst, *line as i32);

            statement.write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;
            let mut chunk = chunk.borrow_mut();
            jmps.push((chunk.write_jump_placeholder(*line as i32), *line));
            match chunk.update_jump(offset) {
                Ok(_) => {}
                Err(_) => {
                    return Err(EvaluateError {
                        error: EvaluateErrorDetails::CodeTooLong,
                        line: *line,
                    });
                }
            }
        }
        let mut chunk = chunk.borrow_mut();

        for i in jmps {
            match chunk.update_jump(i.0) {
                Ok(_) => {}
                Err(_) => {
                    return Err(EvaluateError {
                        error: EvaluateErrorDetails::CodeTooLong,
                        line: i.1,
                    });
                }
            }
        }

        Ok(())
    }
}
