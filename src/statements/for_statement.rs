use crate::{
    compiler::CodeGenerator,
    expressions::{EvaluateError, EvaluateErrorDetails, Expressions, Value},
    statements::{Statement, Statements},
};


#[derive(Debug)]
pub struct ForStatement<'a> {
    pub variable_declare:  Option<Box<Statements<'a>>>,
    pub test: Option<Expressions<'a>>,
    pub inc: Option<Expressions<'a>>,
    pub statement: Box<Statements<'a>>,
    pub begin_line: usize,
    pub end_line: usize,
}

impl ForStatement<'_> {
    pub fn new<'a>(
        variable_declare: Option<Box<Statements<'a>>>,
        test: Option<Expressions<'a>>,
        inc: Option<Expressions<'a>>,
        statement: Box<Statements<'a>>,
        begin_line: usize,
        end_line: usize,
    ) -> ForStatement<'a> {
        ForStatement {
            variable_declare,
            test,
            inc,
            statement,
            begin_line,
            end_line
        }
    }
}

impl<'a> Statement<'a> for ForStatement<'a> {}

impl<'a> CodeGenerator<'a> for ForStatement<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        
        chunk.write_stack_push(self.begin_line as i32);

        let dst = self.dst_or_default(dst_register, &reserved_registers);
        self.variable_declare.as_mut().map(|d| d.write_expression(chunk, Some(dst), reserved_registers.clone()));
        let loop_offset = chunk.code.len();
        match self.test.as_mut() {
            Some(test) => test.write_expression(chunk, Some(dst), reserved_registers.clone()),
            None => { 
                let constant = chunk.get_or_write_constant(Value::Boolean(true), self.begin_line as i32);
                chunk.write_load(dst, constant, self.begin_line as i32);
                Ok(())
            },
        }?;
        let offset =
            chunk.write_jump_if_false_placeholder(dst, self.begin_line as i32);

        self.statement
            .write_expression(chunk, Some(dst), reserved_registers.clone())?;
        
        self.inc.as_mut().map(|d| d.write_expression(chunk, Some(dst), reserved_registers.clone()));
        
        chunk.write_goto(loop_offset as u16, self.end_line as i32);

        match chunk.update_jump(offset) {
            Ok(_) => {
                chunk.write_stack_pop(self.end_line as i32);

            }
            Err(_) => {
                return Err(EvaluateError {
                    error: EvaluateErrorDetails::CodeTooLong,
                    line: self.begin_line,
                });
            }
        }

        Ok(())
    }
}
