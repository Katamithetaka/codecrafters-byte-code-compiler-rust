use std::fmt::Debug;

use crate::{
    compiler::CodeGenerator,
    statements::{
        block_statement::BlockStatement, declare_statement::DeclareStatement,
        expression_statement::ExprStatement, print_statement::PrintStatement,
    },
};

pub mod block_statement;
pub mod declare_statement;
pub mod expression_statement;
pub mod print_statement;

pub trait Statement<'a>: Debug + CodeGenerator<'a> {}

#[derive(derive_more::From, derive_more::TryInto, Debug)]
pub enum Statements<'a> {
    DeclareStatement(DeclareStatement<'a>),
    BlockStatement(BlockStatement<'a>),
    ExprStatement(ExprStatement<'a>),
    PrintStatement(PrintStatement<'a>),
}

impl<'a> Statement<'a> for Statements<'a> {}

impl<'a> CodeGenerator<'a> for Statements<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut crate::compiler::chunk::Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        match self {
            Statements::DeclareStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::BlockStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::ExprStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::PrintStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
        }
    }
}

pub mod prelude {}
