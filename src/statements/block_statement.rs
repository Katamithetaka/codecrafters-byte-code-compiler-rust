use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::{Compiler}},
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
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        chunk.borrow_mut().write_stack_push(self.begin_line as i32);

        for statement in  &mut self.statements {
            if let Statements::FunctionDeclareStatement(func) = statement {
                chunk.borrow_mut().declare_function(func.ident.token, func.ident.line as i32);
                chunk.borrow_mut().write_declare_local(0, func.ident.line as i32);

            }
        }

        for i in &mut self.statements {
            i.write_expression(chunk.clone(), dst_register, reserved_registers.clone())?;
        }
        chunk.borrow_mut().write_stack_pop(self.end_line as i32);

        Ok(())
    }
}
