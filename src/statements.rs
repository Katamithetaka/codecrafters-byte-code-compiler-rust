/// This module defines various statement types and traits used in the interpreter.
/// Statements represent executable units of code in the language being interpreted.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    compiler::{CodeGenerator, compiler::Compiler},
    statements::{
        block_statement::BlockStatement, class_declare_statement::ClassDeclareStatement, declare_statement::DeclareStatement, expression_statement::ExprStatement, for_statement::ForStatement, function_declaration_statement::FunctionDeclareStatement, if_statement::IfStatement, print_statement::PrintStatement, return_statement::ReturnStatement, while_statements::WhileStatement
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
pub mod class_declare_statement;

/// A trait representing a generic statement in the interpreter.
///
/// Statements are executable units of code, such as variable declarations,
/// loops, or function calls. Implementors of this trait must also implement
/// the `CodeGenerator` trait to allow for code generation.
pub trait Statement<'a>: Debug + CodeGenerator<'a> {}

/// Represents the various types of statements in the language.
///
/// Each variant corresponds to a specific type of statement, such as a block,
/// a variable declaration, or a function declaration.
#[derive(derive_more::From, derive_more::TryInto, Debug)]
pub enum Statements<'a> {
    /// A variable declaration statement.
    DeclareStatement(DeclareStatement<'a>),

    /// A block of statements enclosed in braces.
    BlockStatement(BlockStatement<'a>),

    /// An expression statement, which evaluates an expression.
    ExprStatement(ExprStatement<'a>),

    /// A print statement, which outputs the result of an expression.
    PrintStatement(PrintStatement<'a>),

    /// An if statement, which conditionally executes code based on a boolean expression.
    IfStatement(IfStatement<'a>),

    /// A while statement, which executes a block of code as long as a condition is true.
    WhileStatement(WhileStatement<'a>),

    /// A for statement, which iterates over a range or collection.
    ForStatement(ForStatement<'a>),

    /// A function declaration statement.
    FunctionDeclareStatement(FunctionDeclareStatement<'a>),

    /// A return statement, which exits a function and optionally returns a value.
    ReturnStatement(ReturnStatement<'a>),

    ClassDeclareStatement(ClassDeclareStatement<'a>),
}


impl Statements<'_> {
    /// Checks if the statement is a return statement.
    ///
    /// # Returns
    ///
    /// `true` if the statement is a `ReturnStatement`, otherwise `false`.
    pub fn is_return(&self) -> bool {
        match self {
            Self::ReturnStatement(_) => true,
            _ => false
        }
    }
}

impl<'a> Statement<'a> for Statements<'a> {}

impl<'a> CodeGenerator<'a> for Statements<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
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
            },
            Statements::ClassDeclareStatement(statement) => {
                statement.write_expression(chunk, dst_register, reserved_registers)
            }
        }
    }
}

/// The `prelude` module re-exports commonly used types and traits from this module.
///
/// This allows for easier imports in other parts of the codebase.
pub mod prelude {
    pub use super::Statements;
    pub use super::block_statement::BlockStatement;
    pub use super::declare_statement::DeclareStatement;
    pub use super::expression_statement::ExprStatement;
    pub use super::for_statement::ForStatement;
    pub use super::function_declaration_statement::FunctionDeclareStatement;
    pub use super::if_statement::IfStatement;
    pub use super::print_statement::PrintStatement;
    pub use super::return_statement::ReturnStatement;
    pub use super::while_statements::WhileStatement;
    pub use super::class_declare_statement::ClassDeclareStatement;

}
