use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{compiler::{CodeGenerator, compiler::{Compiler}, int_types::{line_type, register_index_type}}, expressions::{Expression, Expressions, get_expression::GetExpression}, value::Value};

#[derive(Debug)]
pub struct SetExpression<'a> {
    pub lhs: GetExpression<'a>,
    pub rhs: Box<Expressions<'a>>,

}

impl<'a> SetExpression<'a> {
    pub fn new(lhs: GetExpression<'a>, rhs: Box<Expressions<'a>>) -> Self {
        Self {
            lhs,
            rhs,
        }
    }
}

impl Display for SetExpression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.lhs, self.rhs)
    }
}

impl<'a> Expression<'a> for SetExpression<'a> {
    fn line_number(&self) -> line_type {
        self.lhs.line_number()
    }
}

impl<'a> CodeGenerator<'a> for SetExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);

        let value_register = self.next_dst(dist, 1, &reserved_registers);
        self.rhs.write_expression(chunk.clone(), Some(value_register), reserved_registers.clone())?;
        self.lhs.lhs.write_expression(chunk.clone(), Some(dist), reserved_registers)?;
        let constant = chunk.borrow_mut().get_or_write_constant(Value::String(self.lhs.rhs.token), self.line_number());
        chunk.borrow_mut().write_set_field(constant, value_register, dist, self.line_number());

        Ok(())
    }
}
