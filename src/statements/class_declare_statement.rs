use std::{cell::RefCell, rc::Rc};

use crate::{
    ParserError, compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}, int_types::{line_type, register_index_type}}, expressions::{
        Value,
        identifier::Identifier,
    }, statements::Statement, value::class::Class
};

#[derive(Debug)]
pub struct ClassDeclareStatement<'a> {
    pub ident: Identifier<'a>,
}
impl<'a> ClassDeclareStatement<'a> {
    pub fn new(
        ident: Identifier<'a>,
    ) -> Self {
        Self {
            ident,
        }
    }
}
impl<'a> Statement<'a> for ClassDeclareStatement<'a> {}

impl<'a> CodeGenerator<'a> for ClassDeclareStatement<'a> {
    fn write_expression(
        &mut self,
        compiler: Rc<RefCell<Compiler<'a>>>,
        dst: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>,
    ) -> crate::compiler::Result {
        let dst_reg = self.dst_or_default(dst, &reserved_registers);
        let mut chunk = compiler.borrow_mut();


        match chunk.declare_variable(self.ident.token, self.ident.line as line_type) {
            Ok(_) => {},
            Err(_) => {
                Err(ParserError {
                    error: crate::ast_parser::ParserErrorDetails::VariableRedeclaration,
                    line: self.ident.line,
                })?
            },
        }

        let constant = chunk.add_constant(Value::Null);

        let class = Class::new(self.ident.token.to_string());

        // todo: handle methods


        chunk.chunk.constants[constant.0 as usize] = Value::Class(class);


        chunk.write_load(dst_reg, constant, self.ident.line as line_type);
        match chunk.resolve_variable(self.ident.token) {
            Ok(ResolvedVar::Local(_)) | Err(_) => {
                chunk.write_declare_local(dst_reg, self.ident.line as line_type);
            },
            Ok(ResolvedVar::Global(varint)) => {
                chunk.write_declare_global(varint, dst_reg, self.ident.line as line_type);
            },
            Ok(ResolvedVar::Upvalue(_)) => unreachable!(),
        }



        Ok(())
    }
}
