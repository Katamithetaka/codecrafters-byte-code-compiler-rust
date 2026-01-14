use crate::{
    compiler::{CodeGenerator, instructions::Instructions},
    expressions::{
        EvaluateError, EvaluateErrorDetails, Function, Value,
        identifier::{Identifier, IdentifierKind},
    },
    statements::{Statement, Statements},
};

#[derive(Debug)]
pub struct FunctionDeclareStatement<'a> {
    pub ident: Identifier<'a>,
    pub args: Vec<Identifier<'a>>,
    pub statements: Vec<Statements<'a>>,
}
impl<'a> FunctionDeclareStatement<'a> {
    pub fn new(
        ident: Identifier<'a>,
        args: Vec<Identifier<'a>>,
        statements: Vec<Statements<'a>>,
    ) -> Self {
        Self {
            ident,
            args,
            statements,
        }
    }
}
impl<'a> Statement<'a> for FunctionDeclareStatement<'a> {}

impl<'a> CodeGenerator<'a> for FunctionDeclareStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst, &reserved_registers);

        match self.ident.kind {
            IdentifierKind::GlobalScope => {
                let constant = chunk
                    .get_or_write_constant(Value::String(self.ident.token), self.ident.line as i32);

                chunk.write_declare_global(constant, dist, self.ident.line as i32);
            }
            IdentifierKind::LocalScope { .. } => {
                chunk.write_declare_local(dist, self.ident.line as i32);
            }
            IdentifierKind::UpperScope { .. } => panic!("tried to declare a upper scope identifier?"),
        };

        let offset = chunk.write_jump_placeholder(self.ident.line as i32);
        let code_begin = chunk.code.len();

        let mut wrote_return = false;
        for statement in &mut self.statements {
            statement.write_expression(chunk, Some(0), vec![])?; // this means that if there is a return statement it'll automatically be in register 0
            
            if statement.is_return() {
                chunk.write_function_return(self.ident.line as i32);
                wrote_return = true;
                break;
            }
        }
        
        if !wrote_return {
            let constant = chunk.get_or_write_constant(Value::Null, self.ident.line as i32);
            chunk.write_load(0, constant, self.ident.line as i32);
            
            
            chunk.write_function_return(self.ident.line as i32);
        }

        match chunk.update_jump(offset) {
            Ok(_) => {
                let function = Function::new(
                    self.ident.token.to_string(),
                    code_begin as u16,
                    self.args.len() as u8,
                );

                let constant = chunk.add_constant(Value::Function(function));
                chunk.write_load(dist, constant, self.ident.line as i32);
                match self.ident.kind {
                    IdentifierKind::GlobalScope => {
                        let constant = chunk.get_or_write_constant(
                            Value::String(self.ident.token),
                            self.ident.line as i32,
                        );

                        chunk.write_set_global(constant, dist, self.ident.line as i32);
                    }
                    IdentifierKind::LocalScope { slot } => {
                        chunk.write_set_local(dist, slot, self.ident.line as i32);
                    }
                    IdentifierKind::UpperScope { .. } => panic!("tried to declare a upper scope identifier?"),
                };
                Ok(())
            }
            Err(_) => {
                return Err(EvaluateError {
                    error: EvaluateErrorDetails::CodeTooLong,
                    line: self.ident.line,
                });
            }
        }
    }
}
