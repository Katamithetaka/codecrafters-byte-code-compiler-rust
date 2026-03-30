use std::{cell::RefCell, rc::Rc};

use crate::{
    ParserError, compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}, garbage_collector::{FunctionKind, GcFunction, HeapObject}, int_types::{line_type, register_index_type}}, expressions::{
        Value,
        identifier::Identifier,
    }, statements::{Statement, Statements}
};

#[derive(Debug)]
pub struct FunctionDeclareStatement<'a> {
    pub ident: Identifier<'a>,
    pub args: Vec<Identifier<'a>>,
    pub statements: Vec<Statements<'a>>,
    pub function_kind: FunctionKind,

}
impl<'a> FunctionDeclareStatement<'a> {
    pub fn new(
        ident: Identifier<'a>,
        args: Vec<Identifier<'a>>,
        statements: Vec<Statements<'a>>,
        function_kind: FunctionKind,
    ) -> Self {
        Self {
            ident,
            args,
            statements,
            function_kind,
        }
    }
}
impl<'a> Statement<'a> for FunctionDeclareStatement<'a> {}

impl<'a> CodeGenerator<'a> for FunctionDeclareStatement<'a> {
    fn write_expression(
        &mut self,
        compiler: Rc<RefCell<Compiler<'a>>>,
        dst: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>,
    ) -> crate::compiler::Result {
        let dst_reg = self.dst_or_default(dst, &reserved_registers);
        let mut chunk = compiler.borrow_mut();

        // Declare the function name in the parent scope
        chunk.declare_function(self.ident.token);
        chunk.mark_declared(self.ident.token.to_string());

        match chunk.resolve_variable(self.ident.token)? {
            ResolvedVar::Local(_) => {

            },
            ResolvedVar::Global(varint) => {
                chunk.write_declare_global(varint, dst_reg, self.ident.line as line_type);
            },
            ResolvedVar::Upvalue(_) => unreachable!(),
        }

        drop(chunk);

        // Create a new nested compiler for the function body
        let fn_compiler = Compiler::with_parent(Rc::clone(&compiler), self.ident.token.to_string(), self.function_kind);



        // Add parameters as locals in the nested compiler
        for arg in &self.args {
            let mut fn_compiler = fn_compiler.borrow_mut();
            match fn_compiler.declare_variable(arg.token) {
                Ok(_) => {},
                Err(_) => {
                    Err(ParserError {
                        error: crate::ast_parser::ParserErrorDetails::VariableRedeclaration,
                        line: self.ident.line,
                    })?
                },
            }
            fn_compiler.locals.last_mut().unwrap().depth = fn_compiler.scope_depth;
        }

        // add this and super
        if let FunctionKind::Method { is_derived } = self.function_kind {
            let mut fn_compiler = fn_compiler.borrow_mut();

            match fn_compiler.declare_variable("this") {
                Ok(_) => {},
                Err(_) => {
                    Err(ParserError {
                        error: crate::ast_parser::ParserErrorDetails::VariableRedeclaration,
                        line: self.ident.line,
                    })?
                },
            }

            let scope_depth = fn_compiler.scope_depth;
            fn_compiler.locals.last_mut().unwrap().depth = scope_depth;

            if is_derived {
                match fn_compiler.declare_variable("super") {
                    Ok(_) => {},
                    Err(_) => {
                        Err(ParserError {
                            error: crate::ast_parser::ParserErrorDetails::VariableRedeclaration,
                            line: self.ident.line,
                        })?
                    },
                }
            }

            let scope_depth = fn_compiler.scope_depth;
            fn_compiler.locals.last_mut().unwrap().depth = scope_depth;

        }
        // Compile the function body in the nested compiler
        let mut wrote_return = false;

        for statement in  &mut self.statements {
            if let Statements::FunctionDeclareStatement(func) = statement {
                let mut fn_compiler = fn_compiler.borrow_mut();

                fn_compiler.declare_function(func.ident.token);
                fn_compiler.write_declare_local(0, func.ident.line as line_type);

            }
        }

        for statement in &mut self.statements {
            statement.write_expression(fn_compiler.clone(), Some(0), vec![])?;

            if statement.is_return() {
                let mut fn_compiler = fn_compiler.borrow_mut();
                fn_compiler.write_function_return(self.ident.line as line_type);
                wrote_return = true;
                break;
            }
        }

        if !wrote_return {
            let mut fn_compiler = fn_compiler.borrow_mut();
            let null_const = fn_compiler.get_or_write_constant(Value::null(), self.ident.line as line_type);
            fn_compiler.write_load(0, null_const, self.ident.line as line_type);
            fn_compiler.write_function_return(self.ident.line as line_type);
        }

        let Compiler { chunk, upvalues, .. } = RefCell::into_inner(Rc::into_inner(fn_compiler).unwrap());

        // Create the function object (with its upvalue count)
        let function = GcFunction {
            name: compiler.borrow().heap().borrow_mut().alloc(HeapObject::String(self.ident.token.to_string())),
            arguments_count: self.args.len() as u8,
            chunk: Box::leak(Box::new(chunk)),
            function_kind: self.function_kind,
        };

        let f = Value::function(compiler.borrow().heap().borrow_mut().alloc(HeapObject::Function(function)));

        let mut compiler = compiler.borrow_mut();
        let constant = compiler.add_constant(f);



        compiler.write_closure(dst_reg, constant, &upvalues, self.ident.line as line_type);

        // Assign the function to the declared variable
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
