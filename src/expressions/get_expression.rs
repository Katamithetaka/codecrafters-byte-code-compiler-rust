use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{compiler::{CodeGenerator, compiler::Compiler, garbage_collector::HeapObject, int_types::{line_type, register_index_type}}, expressions::{Expression, Expressions}, prelude::Identifier, value::Value};

#[derive(Debug)]
pub struct GetExpression<'a> {
    pub lhs: Box<Expressions<'a>>,
    pub rhs: Identifier<'a>,

}

impl<'a> GetExpression<'a> {
    pub fn new(lhs: Box<Expressions<'a>>, rhs: Identifier<'a>) -> Self {
        Self {
            lhs,
            rhs,
        }
    }
}

impl Display for GetExpression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.lhs, self.rhs)
    }
}

impl<'a> Expression<'a> for GetExpression<'a> {
    fn line_number(&self) -> line_type {
        self.lhs.line_number()
    }
}

impl<'a> CodeGenerator<'a> for GetExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);
        self.lhs.write_expression(chunk.clone(), Some(dist), reserved_registers.clone())?;


        let str = HeapObject::String(self.rhs.token.to_string());
        let constant_v = chunk.borrow().heap().borrow_mut().alloc(str);

        let constant = chunk.borrow_mut().get_or_write_constant(Value::string(constant_v), self.line_number());
        chunk.borrow_mut().write_get_field(constant, dist, self.line_number());


        Ok(())
    }
}
