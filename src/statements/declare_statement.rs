use crate::{
    compiler::CodeGenerator,
    expressions::{
        Expression, Expressions, Value,
        identifier::{Identifier, IdentifierKind},
    },
    statements::Statement,
};

#[derive(Debug)]
pub struct DeclareStatement<'a> {
    ident: Identifier<'a>,
    expr: Option<Expressions<'a>>,
}
impl<'a> DeclareStatement<'a> {
    pub fn new(ident: Identifier<'a>, expr: Option<Expressions<'a>>) -> Self {
        Self { ident, expr }
    }
}
impl Statement for DeclareStatement<'_> {}

impl CodeGenerator for DeclareStatement<'_> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        match self.ident.kind {
            IdentifierKind::GlobalScope => {
                let dist = match dst_register {
                    Some(a) => a,
                    None => reserved_registers.iter().max().copied().unwrap_or(0),
                };
                if let Some(expr) = &mut self.expr {
                    expr.write_expression(chunk, Some(dist), reserved_registers)?;
                } else {
                    let constant = chunk.get_or_write_constant(Value::Null, self.ident.line as i32);
                    chunk.write_load(dist, constant, self.ident.line as i32);
                }

                let constant = chunk.get_or_write_constant(
                    Value::String(self.ident.to_string()),
                    self.ident.line as i32,
                );

                chunk.write_declare_global(constant, dist, self.ident.line as i32);

                Ok(())
            }
            _ => unimplemented!(),
        }
    }
}
