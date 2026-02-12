use std::{cell::RefCell, rc::Rc};

use strum::ParseError;

use crate::{
    ParserError, compiler::{CodeGenerator, compiler::Compiler}, expressions::{Expressions, Value}, statements::Statement
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
        chunk: Rc<RefCell<Compiler<'a>>>,
        _dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = 0;

        if let None = chunk.borrow().enclosing {
            Err(ParserError {
                error: crate::ast_parser::ParserErrorDetails::InvalidReturnStatement,
                line: self.line_number,
            })?
        }

        if let Some(expr) = &mut self.expr {
            expr
                .write_expression(chunk.clone(), Some(dist), reserved_registers)?;
        }
        else {
            let mut chunk = chunk.borrow_mut();
            let constant = chunk.get_or_write_constant(Value::Null, self.line_number as i32);
            chunk.write_load(0, constant, self.line_number as i32);
        }
        let mut chunk = chunk.borrow_mut();

        chunk.write_function_return( self.line_number as i32);

        Ok(())
    }


}
