use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::{Compiler, ResolvedVar}},
    expressions::{
        Expressions, Value,
        identifier::Identifier,
    },
    statements::Statement,
};

#[derive(Debug)]
pub struct DeclareStatement<'a> {
    pub ident: Identifier<'a>,
    pub expr: Option<Expressions<'a>>,
}
impl<'a> DeclareStatement<'a> {
    pub fn new(ident: Identifier<'a>, expr: Option<Expressions<'a>>) -> Self {
        Self { ident, expr }
    }
}
impl<'a> Statement<'a> for DeclareStatement<'a> {}

impl<'a> CodeGenerator<'a> for DeclareStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst, &reserved_registers);



        chunk.borrow_mut().declare_variable(self.ident.token, self.ident.line as i32);

        if let Some(expr) = &mut self.expr {
            expr.write_expression(chunk.clone(), Some(dist), reserved_registers)?;
        } else {
            let constant = chunk.borrow_mut().get_or_write_constant(Value::Null, self.ident.line as i32);
            chunk.borrow_mut().write_load(dist, constant, self.ident.line as i32);
        };

        let mut chunk = chunk.borrow_mut();
        match chunk.resolve_variable(self.ident.token) {
            Ok(ResolvedVar::Local(_)) | Err(_) => {
                chunk.write_declare_local(dist, self.ident.line as i32);

            },
            Ok(ResolvedVar::Global(varint)) => {
                chunk.write_declare_global(varint, dist, self.ident.line as i32);
            },
            Ok(ResolvedVar::Upvalue(_)) => unreachable!(),
        }

        Ok(())
    }
}
