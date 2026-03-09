use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};


use crate::{
    ParserError, compiler::{CodeGenerator, compiler::Compiler, int_types::{line_type, register_index_type}}, expressions::{
        assignment_expression::AssignmentExpression, binary_expression::BinaryExpression, call_expression::CallExpression, equality_expression::EqualityExpression, get_expression::GetExpression, group::Group, identifier::Identifier, literal::Literal, logical_expression::LogicalExpression, relation_expression::RelationalExpression, set_expression::SetExpression, super_expression::Super, this_expression::This, unary_expression::UnaryExpression
    }
};

pub use crate::value::*;

/// Module containing the definition and implementation of assignment expressions.
pub mod assignment_expression;
/// Module containing the definition and implementation of binary expressions.
pub mod binary_expression;
/// Module containing the definition and implementation of equality expressions.
pub mod equality_expression;
/// Module containing the definition and implementation of group expressions.
pub mod group;
/// Module containing the definition and implementation of identifiers.
pub mod identifier;
/// Module containing the definition and implementation of literal values.
pub mod literal;
/// Module containing the definition and implementation of logical expressions.
pub mod logical_expression;
/// Module containing the definition and implementation of relational expressions.
pub mod relation_expression;
/// Module containing the definition and implementation of unary expressions.
pub mod unary_expression;
/// Module containing the definition and implementation of call expressions.
pub mod call_expression;
pub mod get_expression;
pub mod set_expression;
pub mod this_expression;
pub mod super_expression;

/// The `prelude` module re-exports commonly used types and functions from this module.
///
/// This allows for easier imports in other parts of the codebase.
pub mod prelude {
    pub use super::{
        assignment_expression::AssignmentExpression,
        binary_expression::{BinaryExpression, BinaryOp},
        equality_expression::{EqualityExpression, EqualityOp},
        group::Group,
        identifier::Identifier,
        literal::Literal,
        logical_expression::{LogicalExpression, LogicalOp},
        relation_expression::{RelationalExpression, RelationalOp},
        unary_expression::{UnaryExpression, UnaryOp},
        call_expression::CallExpression,
        Expressions, Expression, EvaluateError, EvaluateErrorDetails, Value,
    };
}


/// Enum representing the various types of errors that can occur during expression evaluation.
#[derive(thiserror::Error, Debug)]
pub enum EvaluateErrorDetails {
    /// Error for encountering an unexpected return statement.
    #[error("Unexpected return statement")]
    UnexpectedReturn,
    /// Error for encountering an unknown operation code.
    #[error("Unknown Operation {0}")]
    UnexpectedOpCode(u8),
    /// General evaluation error with a message.
    #[error("Evaluate error: {0}")]
    Error(String),
    /// Error for expecting a value but finding none.
    #[error("Expected value from expression, got None")]
    ExpectedValue,
    /// Error for binary operations requiring numeric operands.
    #[error("Operands must be numbers.")]
    BinaryNumberOp,
    /// Error for expecting a numeric operand.
    #[error("Operand must be number.")]
    ExpectedNumber,
    /// Error for expecting a string operand.
    #[error("Operand must be string.")]
    ExpectedString,
    /// Error for expecting a function operand.
    #[error("Operand must be function.")]
    ExpectedFunction,

    #[error("Operand must be class instance.")]
    ExpectedClassInstance,
    /// Error for mismatched operand types in binary operations.
    #[error("Operands must be two numbers or two strings. ")]
    UnmatchedTypes,
    /// Error for referencing an undefined variable.
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    /// Error for expecting an identifier to be a string but finding otherwise.
    #[error("Expected identifier to be a string, but it wasn't")]
    InvalidIdentifierType,
    /// Error for defining a local variable in the global scope.
    #[error("Tried to define a local variable in global scope")]
    LocalInGlobal,
    /// Error for attempting to pop the stack in the global scope.
    #[error("Tried to pop stack in global scope.")]
    InvalidStackPop,
    /// Error for exceeding the maximum number of local variables in a scope.
    #[error("Stack overflow: Too many locals defined in local scopes")]
    StackOverflow,
    /// Error for providing an invalid number of arguments to a function.
    #[error("Invalid number of arguments when calling function!")]
    InvalidArgCount,
    /// Error for calling a function without pushing to the call stack.
    #[error("Call stack wasn't pushed before calling a function!")]
    CallStackEmpty,
    /// Error for encountering an invalid return statement.
    #[error("Invalid return statement")]
    InvalidReturnStatement,
    /// Error for a jump statement exceeding the boundaries of a `u16`.
    #[error("Jump statement didn't fit in the boundaries of a u16")]
    CodeTooLong,
    /// Error for failing to fetch stdin during a debug break.
    #[error("Debug break couldn't fetch stdin")]
    StdinFailed,
    #[error("Tried to access an upvalue when no call stack was available!")]
    InvalidUpvalueAccess,
    #[error("Upvalue was not a shared ptr!")]
    InvalidUpvalueType,

    #[error("{0}")]
    ParserError( #[from] ParserError),

    #[error("Method was unbinded ('this' was not set)")]
    UnbindedMethod

}

#[derive(thiserror::Error, Debug)]
pub struct EvaluateError {
    pub error: EvaluateErrorDetails,
    pub line: line_type,
}

impl From<ParserError> for EvaluateError {
    fn from(err: ParserError) -> Self {
        EvaluateError {
            line: err.line as line_type,
            error: EvaluateErrorDetails::ParserError(err),
        }
    }
}



impl Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.error)
    }
}

pub trait Expression<'a>: Display + Debug + CodeGenerator<'a> {
    fn line_number(&self) -> line_type;
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::TryInto)]
pub enum Expressions<'a> {
    #[from]
    BinaryExpression(BinaryExpression<'a>),
    #[from]
    EqualityExpression(EqualityExpression<'a>),
    #[from]
    LogicalExpression(LogicalExpression<'a>),
    #[from]
    Group(Group<'a>),
    #[from]
    Identifier(Identifier<'a>),
    #[from]
    Literal(Literal<'a>),
    #[from]
    UnaryExpression(UnaryExpression<'a>),
    #[from]
    RelationalExpression(RelationalExpression<'a>),
    #[from]
    AssignmentExpression(AssignmentExpression<'a>),
    #[from]
    GetExpression(GetExpression<'a>),
    #[from]
    SetExpression(SetExpression<'a>),
    #[from]
    CallExpression(CallExpression<'a>),
    #[from]
    ThisExpression(This<'a>),
    #[from]
    SuperExpression(Super<'a>)
}

impl<'a> Expression<'a> for Expressions<'a> {
    fn line_number(&self) -> line_type {
        match self {
            Expressions::BinaryExpression(binary_expression) => binary_expression.line_number(),
            Expressions::EqualityExpression(equality_expression) => {
                equality_expression.line_number()
            }
            Expressions::LogicalExpression(expression) => expression.line_number(),
            Expressions::Group(group) => group.line_number(),
            Expressions::Identifier(identifier) => identifier.line_number(),
            Expressions::Literal(literal) => literal.line_number(),
            Expressions::UnaryExpression(unary_expression) => unary_expression.line_number(),
            Expressions::RelationalExpression(relation_expression) => {
                relation_expression.line_number()
            }
            Expressions::AssignmentExpression(assignmpent_expression) => {
                assignmpent_expression.line_number()
            }
            Expressions::CallExpression(call_expression) =>  {
                call_expression.line_number()
            }
            Expressions::GetExpression(get_expression) => {
                get_expression.line_number()
            }
            Expressions::SetExpression(get_expression) => {
                get_expression.line_number()
            },
            Expressions::ThisExpression(this) => this.line_number(),
            Expressions::SuperExpression(this) => this.line_number(),

        }
    }
}

impl<'a> CodeGenerator<'a> for Expressions<'a> {
    fn write_expression(
        &mut self,
        chunk: Rc<RefCell<Compiler<'a>>>,
        dst_register: Option<register_index_type>,
        reserved_registers: Vec<register_index_type>
    ) -> crate::compiler::Result {
        match self {
            Expressions::BinaryExpression(binary_expression) => {
                binary_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::EqualityExpression(equality_expression) => {
                equality_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::LogicalExpression(expression) => {
                expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::Group(group) => {
                group.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::Identifier(identifier) => {
                identifier.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::Literal(literal) => {
                literal.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::UnaryExpression(unary_expression) => {
                unary_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::RelationalExpression(relation_expression) => {
                relation_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::AssignmentExpression(assignment_expression) => {
                assignment_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::CallExpression(call_expression) => {
                call_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Self::GetExpression(get_expression) => {
                get_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Self::SetExpression(get_expression) => {
                get_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Self::ThisExpression(this) => {
                this.write_expression(chunk, dst_register, reserved_registers)
            }
            Self::SuperExpression(this) => {
                this.write_expression(chunk, dst_register, reserved_registers)
            }
        }
    }
}
