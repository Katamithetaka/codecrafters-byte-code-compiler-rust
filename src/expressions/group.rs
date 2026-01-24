use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler},
    expressions::{Expression, Expressions},
};

#[derive(Debug)]
pub struct Group<'a> {
    pub expr: Box<Expressions<'a>>,
}

impl<'a> Display for Group<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(group {})", self.expr)
    }
}

impl<'a> Group<'a> {
    pub fn new(expr: Box<Expressions<'a>>) -> Self {
        return Self { expr };
    }
}

impl<'a> Expression<'a> for Group<'a> {
    fn line_number(&self) -> usize {
        self.expr.line_number()
    }
}

impl<'a> CodeGenerator<'a> for Group<'a> {
    fn write_expression(
        &mut self,
        compiler: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        self.expr
            .write_expression(compiler, dst_register, reserved_registers)
    }
}
