use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    Token, compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}, garbage_collector::HeapObject, int_types::{line_type, register_index_type}}, expressions::Expression, prelude::{EvaluateError, Identifier}, value::{EvaluateErrorDetails, Value}
};



#[derive(Debug)]
pub struct Super<'a> {
    pub token: &'a str,
    pub line: line_type,
    pub identifier: Identifier<'a>
}

impl<'a> Display for Super<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl<'a> Super<'a> {
    pub fn new(token: &'a Token<'a>, identifier: Identifier<'a>) -> Self {
        return Self {
            token: token.lexeme,
            line: token.line as line_type,
            identifier
        };
    }
}

impl<'a> Expression<'a> for Super<'a> {
    fn line_number(&self) -> line_type{
        self.line
    }
}

impl<'a> CodeGenerator<'a> for Super<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);

        if !chunk.borrow().is_in_derived_method() {
            return Err(EvaluateError {
                error: EvaluateErrorDetails::ParserError(crate::ParserError { error: crate::ast_parser::ParserErrorDetails::InvalidSuperUsage, line: self.line_number() }),
                line: self.line_number()
            })
        }

        let mut chunk = chunk.borrow_mut();

        let str = HeapObject::String(self.identifier.token.to_string());
        let constant_v = chunk.heap().borrow_mut().alloc(str);

        let constant = chunk.get_or_write_constant(Value::string(constant_v), self.line_number());

        let value_register = self.next_dst(dst, 1, &reserved_registers);


        chunk.write_load(value_register, constant, self.identifier.line);



        let this_register = self.next_dst(dst, 3, &reserved_registers);

        match chunk.resolve_variable("this")? {
            ResolvedVar::Global(slot) => {
                chunk.write_get_global(slot, this_register, self.line as line_type);
            }
            ResolvedVar::Local(slot) => {
                eprintln!("Resolved this as {slot}");
                chunk.write_get_local(this_register, slot, self.line as line_type);
            }
            ResolvedVar::Upvalue(slot) => {
                eprintln!("Resolved this as upvalue {slot}");

                chunk.write_get_upvalue(this_register, slot, self.line as line_type);
            }
        }

        let super_register = self.next_dst(dst, 2, &reserved_registers);

        match chunk.resolve_variable(self.token)? {
            ResolvedVar::Global(slot) => {
                chunk.write_get_global(slot, super_register, self.line as line_type);
            }
            ResolvedVar::Local(slot) => {
                eprintln!("Resolved super as {slot}");

                chunk.write_get_local(super_register, slot, self.line as line_type);
            }
            ResolvedVar::Upvalue(slot) => {
                eprintln!("Resolved super as upvalue {slot}");

                chunk.write_get_upvalue(super_register, slot, self.line as line_type);
            }
        }


        chunk.write_super(value_register, super_register, this_register, dst, self.line);

        Ok(())
    }
}
