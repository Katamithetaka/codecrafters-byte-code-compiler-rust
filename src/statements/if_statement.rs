use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler, int_types::{line_type, register_index_type}},
    expressions::{Expressions, Value},
    statements::{Statement, Statements},
};

#[derive(Debug)]
pub struct IfStatement<'a> {
    pub statements: Vec<(Option<Expressions<'a>>, Statements<'a>, line_type)>,
}

impl IfStatement<'_> {
    pub fn new<'a>(
        statements: Vec<(Option<Expressions<'a>>, Statements<'a>, line_type)>,
    ) -> IfStatement<'a> {
        IfStatement { statements }
    }
}

impl<'a> Statement<'a> for IfStatement<'a> {}

impl<'a> CodeGenerator<'a> for IfStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
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
                    let constant = chunk.get_or_write_constant(Value::bool(true), *line as line_type);
                    chunk.write_load(dst, constant, *line as line_type);
                }
            };


            let offset = chunk.borrow_mut().write_jump_if_false_placeholder(dst, *line as line_type)?;

            statement.write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;
            let mut chunk = chunk.borrow_mut();
            jmps.push((chunk.write_jump_placeholder(*line as line_type)?, *line));
            chunk.update_jump(offset)?;
        }
        let mut chunk = chunk.borrow_mut();

        for i in jmps {
            chunk.update_jump(i.0)?;
        }

        Ok(())
    }
}
