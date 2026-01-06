use crate::compiler::chunk::Chunk;

pub mod chunk;
pub mod instructions;
pub mod value;
pub mod varint;
pub mod vm;

pub const BINARY_FORMAT: u8 = 1;
pub type Result = std::result::Result<(), crate::expressions::EvaluateError>;
pub trait CodeGenerator<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> Result;

    fn dst_or_default(&self, dst: Option<u8>, reserved_registers: &[u8]) -> u8 {
        dst.unwrap_or(reserved_registers.iter().max().copied().unwrap_or(0) + 1)
    }
}

pub mod macros {
    macro_rules! binary_op {
        ($instruction: ident, $dst_register: ident, $reserved_registers: ident, $chunk: ident, $self: ident) => {
            {
                let my_dst_register_0 = match $dst_register {
                    Some(v) => v,
                    None => $reserved_registers.iter().max().copied().unwrap_or(0) + 1, // this can be assumed to never happen
                };

                let reserved_0 = $reserved_registers.clone();

                let mut reserved_1 = $reserved_registers.clone();
                reserved_1.push(my_dst_register_0);
                let my_dst_register_1 = (reserved_1.iter().max().copied().unwrap_or(0) + 1);

                $self.lhs
                    .write_expression($chunk, Some(my_dst_register_0), reserved_0)?;

                $self.rhs
                    .write_expression($chunk, Some(my_dst_register_1), reserved_1)?;

                let dst = match $dst_register {
                    Some(dst) => dst,
                    None => my_dst_register_0,
                };
                let r0 = my_dst_register_0;
                let r1 = my_dst_register_1;

                $chunk.write_binary($instruction, r0, r1, dst, $self.line_number() as i32);
                Ok(())
            }
        };
    }

    pub(crate) use binary_op;
}
