use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{compiler::{CodeGenerator, compiler::{Compiler, Local}}, expressions::{Expression, Expressions}};

#[derive(Debug)]
pub struct CallExpression<'a> {
    pub lhs: Box<Expressions<'a>>,
    pub arguments: Vec<Expressions<'a>>,

}

impl<'a> CallExpression<'a> {
    pub fn new(lhs: Box<Expressions<'a>>, arguments: Vec<Expressions<'a>>) -> Self {
        Self {
            lhs,
            arguments,
        }
    }
}

impl Display for CallExpression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.lhs)?;
        for argument in &self.arguments {
            write!(f, "{argument},")?;
        }
        write!(f, ")")
    }
}

impl<'a> Expression<'a> for CallExpression<'a> {
    fn line_number(&self) -> usize {
        self.lhs.line_number()
    }
}

impl<'a> CodeGenerator<'a> for CallExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);

        let dst = self.next_dst(dist, 1, &reserved_registers);
        self.lhs.write_expression(chunk.clone(), Some(dist), reserved_registers.clone())?;


        for argument in self.arguments.iter_mut() {
            argument.write_expression(chunk.clone(), Some(dst), reserved_registers.clone())?;
            chunk.borrow_mut().locals.push(Local { name: "".to_string(), depth: 0, is_captured: false, is_predeclared: false });
            chunk.borrow_mut().write_declare_local(dst, argument.line_number() as i32);
        }
        chunk.borrow_mut().write_fn_call(dist, self.arguments.len() as u8, self.lhs.line_number() as i32);


        Ok(())
    }
}
