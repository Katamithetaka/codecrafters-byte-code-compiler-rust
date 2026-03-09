use std::{cell::RefCell, rc::Rc};

use crate::{
    ParserError, compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}, int_types::{line_type, register_index_type}}, expressions::{
        Value,
        identifier::Identifier,
    }, prelude::FunctionDeclareStatement, statements::Statement, value::class::Class
};

#[derive(Debug)]
pub struct ClassDeclareStatement<'a> {
    pub ident: Identifier<'a>,
    pub functions: Vec<FunctionDeclareStatement<'a>>,
    pub inherited_class: Option<Identifier<'a>>
}
impl<'a> ClassDeclareStatement<'a> {
    pub fn new(
        ident: Identifier<'a>,
        functions: Vec<FunctionDeclareStatement<'a>>,
        inherited_class: Option<Identifier<'a>>
    ) -> Self {
        Self {
            ident,
            functions,
            inherited_class
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



        compiler.borrow_mut().declare_function(self.ident.token, self.ident.line as line_type);
        compiler.borrow_mut().mark_declared(self.ident.token.to_string());

        let mut c = compiler.borrow_mut();
        match c.resolve_variable(self.ident.token)? {
            ResolvedVar::Local(_) => {
                c.write_declare_local(0, self.ident.line as line_type);

            },
            ResolvedVar::Global(varint) => {
                c.write_declare_global(varint, dst_reg, self.ident.line as line_type);
            },
            ResolvedVar::Upvalue(_) => unreachable!(),
        }
        drop(c);

        let constant = compiler.borrow_mut().add_constant(Value::Null);


        let class = Class::new(self.ident.token.to_string());
        let func_dst = self.next_dst(1, dst_reg, &reserved_registers);


        compiler.borrow_mut().chunk.constants[constant.0 as usize] = Value::Class(class);

        compiler.borrow_mut().write_load(dst_reg, constant, self.ident.line as line_type);

        compiler.borrow_mut().write_stack_push(self.ident.line);

        eprintln!("Got here 1!");
        for func in self.functions.iter_mut() {
            func.function_kind = crate::value::callable::FunctionKind::Method;
            compiler.borrow_mut().declare_function(func.ident.token, func.ident.line as line_type);
            compiler.borrow_mut().write_declare_local(0, func.ident.line as line_type);

            func.write_expression(compiler.clone(), Some(func_dst), reserved_registers.clone())?;

            compiler.borrow_mut().write_method_declare(func_dst, dst_reg, self.ident.line as line_type);

        }
        eprintln!("Got here 2!");

        if let Some(inherited) = &self.inherited_class {

            if inherited.token == self.ident.token {
                Err(ParserError {
                    error: crate::ast_parser::ParserErrorDetails::InvalidInheritance,
                    line: inherited.line,
                })?
            }
            let mut compiler = compiler.borrow_mut();

            match compiler.resolve_variable(inherited.token)? {
                    ResolvedVar::Local(slot) => {
                        compiler.write_get_local(func_dst, slot, inherited.line as line_type);
                    },
                    ResolvedVar::Global(varint) => {compiler.write_get_global(varint, func_dst, inherited.line as line_type);},
                    ResolvedVar::Upvalue(slot) => { compiler.write_get_upvalue(func_dst, slot, inherited.line);},
            }

            compiler.write_inherit_methods(func_dst, dst_reg, self.ident.line);
        }



        compiler.borrow_mut().write_stack_pop(self.ident.line);

        let mut compiler = compiler.borrow_mut();

        match compiler.resolve_variable(self.ident.token)? {
                ResolvedVar::Local(slot) => {
                    compiler.write_set_local(dst_reg, slot, self.ident.line as line_type);
                },
                ResolvedVar::Global(varint) => compiler.write_set_global(varint, dst_reg, self.ident.line as line_type),
                ResolvedVar::Upvalue(_) => unreachable!(),
        }





        Ok(())
    }
}
