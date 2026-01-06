use crate::{
    compiler::CodeGenerator,
    expressions::{EvaluateError, EvaluateErrorDetails, Expressions, Value},
    statements::{Statement, Statements},
};

#[derive(Debug)]
pub struct IfStatement<'a> {
    pub statements: Vec<(Option<Expressions<'a>>, Statements<'a>, usize)>,
}

impl IfStatement<'_> {
    pub fn new<'a>(
        statements: Vec<(Option<Expressions<'a>>, Statements<'a>, usize)>,
    ) -> IfStatement<'a> {
        IfStatement { statements }
    }
}

impl<'a> Statement<'a> for IfStatement<'a> {}

impl<'a> CodeGenerator<'a> for IfStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dst = self.dst_or_default(dst_register, &reserved_registers);
        for (expr, statement, line) in self.statements.iter_mut() {
            match expr {
                Some(expr) => {
                    expr.write_expression(chunk, Some(dst), reserved_registers.clone())?
                }
                None => {
                    let constant = chunk.get_or_write_constant(Value::Boolean(true), *line as i32);
                    chunk.write_load(dst, constant, *line as i32);
                }
            };

            let offset = chunk.write_jump_if_false_placeholder(dst, *line as i32);
            statement.write_expression(chunk, Some(dst), reserved_registers.clone())?;
            match chunk.update_jump_if_false(offset) {
                Ok(_) => {}
                Err(_) => {
                    return Err(EvaluateError {
                        error: EvaluateErrorDetails::CodeTooLong,
                        line: *line,
                    });
                }
            }
        }

        Ok(())
    }
}
