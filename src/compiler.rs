use crate::compiler::chunk::Chunk;

pub mod chunk;
pub mod instructions;
pub mod value;
pub mod varint;
pub mod vm;

/// The binary format version used by the compiler.
/// This constant defines the version of the binary format that the compiler generates.
pub const BINARY_FORMAT: u8 = 1;
/// A type alias for the result type used in the compiler.
/// Represents either a successful operation or an `EvaluateError`.
pub type Result = std::result::Result<(), crate::expressions::EvaluateError>;
/// A trait for generating code from expressions and statements.
///
/// This trait defines methods for writing expressions to a `Chunk` of bytecode.
pub trait CodeGenerator<'a> {
    /// Writes an expression to the given `Chunk` of bytecode.
    ///
    /// # Arguments
    ///
    /// * `chunk` - The `Chunk` to write the bytecode to.
    /// * `dst_register` - The destination register for the result of the expression.
    /// * `reserved_registers` - A list of registers that are reserved and cannot be used.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `EvaluateError`.
    fn write_expression(
        &mut self,
        chunk: &mut Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> Result;

    /// Determines the destination register to use, defaulting to the next available register.
    ///
    /// # Arguments
    ///
    /// * `dst` - An optional destination register.
    /// * `reserved_registers` - A list of registers that are reserved and cannot be used.
    ///
    /// # Returns
    ///
    /// The destination register to use.
    fn dst_or_default(&self, dst: Option<u8>, reserved_registers: &[u8]) -> u8 {
        dst.unwrap_or(reserved_registers.iter().max().copied().unwrap_or(0) + 1)
    }
    
    /// Calculates the next destination register based on the current destination and an offset.
    ///
    /// # Arguments
    ///
    /// * `dst` - The current destination register.
    /// * `offset` - The offset to apply to the destination register.
    /// * `reserved_registers` - A list of registers that are reserved and cannot be used.
    ///
    /// # Returns
    ///
    /// The next destination register.
    fn next_dst(&self, dst: u8, offset: u8, reserved_registers: &[u8]) -> u8 {
        reserved_registers.iter().max().copied().unwrap_or(0) + dst + offset
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
