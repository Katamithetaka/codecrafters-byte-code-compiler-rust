use crate::{
    compiler::CodeGenerator,
    expressions::{
        Expressions, Value,
        identifier::{Identifier, IdentifierKind},
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
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst, &reserved_registers);

        if let Some(expr) = &mut self.expr {
            expr.write_expression(chunk, Some(dist), reserved_registers)?;
        } else {
            let constant = chunk.get_or_write_constant(Value::Null, self.ident.line as i32);
            chunk.write_load(dist, constant, self.ident.line as i32);
        }
        match self.ident.kind {
            IdentifierKind::GlobalScope => {
                let constant = chunk
                    .get_or_write_constant(Value::String(self.ident.token), self.ident.line as i32);

                chunk.write_declare_global(constant, dist, self.ident.line as i32);

                Ok(())
            }
            IdentifierKind::LocalScope { .. } => {
                chunk.write_declare_local(dist, self.ident.line as i32);
                Ok(())
            }
            IdentifierKind::UpperScope { .. } => panic!("tried to declare a upper scope identifier?"),
        }
    }
}
