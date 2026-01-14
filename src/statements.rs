use std::fmt::Debug;

use crate::{
    compiler::CodeGenerator,
    statements::{
        block_statement::BlockStatement, declare_statement::DeclareStatement, expression_statement::ExprStatement, for_statement::ForStatement, function_declaration_statement::FunctionDeclareStatement, if_statement::IfStatement, print_statement::PrintStatement, return_statement::ReturnStatement, while_statements::WhileStatement
    },
};

pub mod block_statement;
pub mod declare_statement;
pub mod expression_statement;
pub mod if_statement;
pub mod print_statement;
pub mod while_statements;
pub mod for_statement;
pub mod function_declaration_statement;
pub mod return_statement;

pub trait Statement<'a>: Debug + CodeGenerator<'a> {}

#[derive(derive_more::From, derive_more::TryInto, Debug)]
pub enum Statements<'a> {
    DeclareStatement(DeclareStatement<'a>),
    BlockStatement(BlockStatement<'a>),
    ExprStatement(ExprStatement<'a>),
    PrintStatement(PrintStatement<'a>),
    IfStatement(IfStatement<'a>),
    WhileStatement(WhileStatement<'a>),
    ForStatement(ForStatement<'a>),
    FunctionDeclareStatement(FunctionDeclareStatement<'a>),
    ReturnStatement(ReturnStatement<'a>),
    
}

impl Statements<'_> {
    pub fn is_return(&self) -> bool {
        match self {
            _ => false
        }
    }
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
            Statements::IfStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::WhileStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::ForStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::FunctionDeclareStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
            Statements::ReturnStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
        }
    }
}

pub mod prelude {}
