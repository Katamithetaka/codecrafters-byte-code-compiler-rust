use std::fmt::Display;

use crate::{compiler::CodeGenerator, expressions::{Expression, Expressions}};

#[derive(Debug)]
pub struct CallExpression<'a> {
    pub lhs: Box<Expressions<'a>>,
    pub arguments: Vec<Expressions<'a>>,

}

impl<'a> CallExpression<'a> {
    pub fn new(lhs: Box<Expressions<'a>>, arguments: Vec<Expressions<'a>>) -> Self {
        Self {
            lhs,
            arguments,
        }
    }
}

impl Display for CallExpression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.lhs)?;
        for argument in &self.arguments {
            write!(f, "{argument},")?;
        }
        write!(f, ")")
    }
}

impl<'a> Expression<'a> for CallExpression<'a> {
    fn line_number(&self) -> usize {
        self.lhs.line_number()
    }
}

impl<'a> CodeGenerator<'a> for CallExpression<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let dist = self.dst_or_default(dst_register, &reserved_registers);
    
        let dst = self.next_dst(dist, 1, &reserved_registers);
        
        
        for argument in &mut self.arguments {
            argument.write_expression(chunk, Some(dst), reserved_registers.clone())?;
            chunk.write_declare_local(dst, argument.line_number() as i32);
        }
        chunk.write_call_stack_push(dist, self.lhs.line_number() as i32);
        self.lhs.write_expression(chunk, Some(dist), reserved_registers.clone())?;
        
        chunk.write_fn_call(dist, self.arguments.len() as u8, self.lhs.line_number() as i32);
        
        
        Ok(())
    }
}