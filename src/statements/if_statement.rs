use crate::{
    compiler::{CodeGenerator, instructions::Instructions},
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
        let dst_1 = dst + 1;
        let dst_2 = dst_1 + 1;
        let mut reserved = reserved_registers.clone();
        reserved.push(dst_1);

        let mut current_i = 0;

        for (expr, statement, line) in self.statements.iter_mut() {
            if current_i == 0 {
                let constant = chunk.get_or_write_constant(Value::Boolean(false), *line as i32);
                chunk.write_load(dst_1, constant, *line as i32);
            }

            chunk.write_unary(Instructions::Bang, dst_1, dst_1, *line as i32);

            match expr {
                Some(expr) => {
                    expr.write_expression(chunk, Some(dst_2), reserved_registers.clone())?
                }
                None => {
                    let constant = chunk.get_or_write_constant(Value::Boolean(true), *line as i32);
                    chunk.write_load(dst_2, constant, *line as i32);
                }
            };

            chunk.write_binary(Instructions::And, dst_1, dst_2, dst_1, *line as i32);

            let offset = chunk.write_jump_if_false_placeholder(dst_1, *line as i32);

            statement.write_expression(chunk, Some(dst), reserved.clone())?;
            match chunk.update_jump_if_false(offset) {
                Ok(_) => {
                    current_i += 1;
                }
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
