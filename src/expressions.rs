use std::fmt::{Debug, Display};

use crate::{
    compiler::{CodeGenerator, chunk::Chunk},
    expressions::{
        assignment_expression::AssignmentExpression, binary_expression::BinaryExpression,
        equality_expression::EqualityExpression, group::Group, identifier::Identifier,
        literal::Literal, relation_expression::RelationalExpression,
        unary_expression::UnaryExpression,
    },
};

pub mod assignment_expression;
pub mod binary_expression;
pub mod equality_expression;
pub mod group;
pub mod identifier;
pub mod literal;
pub mod relation_expression;
pub mod unary_expression;

#[derive(Clone, PartialEq, Debug)]
pub enum Value<S> {
    Number(f64),
    String(S),
    Null,
    Boolean(bool),
}

impl<S: Display> Display for Value<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "{}", s),
            Value::Null => f.write_str("nil"),
            Value::Boolean(s) => write!(f, "{}", s),
        }
    }
}

impl<S> Value<S> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(v) => *v,
            _ => true,
        }
    }
}

impl<'a> From<Value<&'a str>> for Value<String> {
    fn from(value: Value<&'a str>) -> Self {
        match value {
            Value::Number(a) => Value::Number(a),
            Value::String(s) => Value::String(s.to_string()),
            Value::Null => Value::Null,
            Value::Boolean(b) => Value::Boolean(b),
        }
    }
}

impl<S> Value<S> {
    pub fn as_number(&self) -> Result<f64, EvaluateErrorDetails> {
        match self {
            Value::Number(a) => Ok(*a),
            _ => Err(EvaluateErrorDetails::ExpectedNumber),
        }
    }

    pub fn as_string(&self) -> Result<&S, EvaluateErrorDetails> {
        match self {
            Value::String(a) => Ok(a),
            _ => Err(EvaluateErrorDetails::ExpectedString),
        }
    }

    pub fn as_ident(&self) -> Result<&S, EvaluateErrorDetails> {
        return self
            .as_string()
            .or(Err(EvaluateErrorDetails::InvalidIdentifierType));
    }

    pub fn as_binary_number_op(&self) -> Result<f64, EvaluateErrorDetails> {
        return self
            .as_number()
            .or(Err(EvaluateErrorDetails::BinaryNumberOp));
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EvaluateErrorDetails {
    #[error("Unexpected return statement")]
    UnexpectedReturn,
    #[error("Unknown Operation {0}")]
    UnexpectedOpCode(u8),
    #[error("Evaluate error: {0}")]
    Error(String),
    #[error("Expected value from expressio, got None")]
    ExpectedValue,
    #[error("Operands must be numbers.")]
    BinaryNumberOp,
    #[error("Operand must be number.")]
    ExpectedNumber,
    #[error("Operand must be string.")]
    ExpectedString,
    #[error("Operands must be two numbers or two strings. ")]
    UnmatchedTypes,
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Expected identifier to be a string, but it wasn't")]
    InvalidIdentifierType,
    #[error("Tried to define a local variable in global scope")]
    LocalInGlobal,
    #[error("Tried to pop stack in global scope.")]
    InvalidStackPop,
    #[error("Stack overflow: Too many locals defined in local scopes")]
    StackOverflow,
}

#[derive(thiserror::Error, Debug)]
pub struct EvaluateError {
    pub error: EvaluateErrorDetails,
    pub line: usize,
}

impl Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.error)
    }
}

pub trait Expression<'a>: Display + Debug + CodeGenerator<'a> {
    fn line_number(&self) -> usize;
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::TryInto)]
pub enum Expressions<'a> {
    #[from]
    BinaryExpression(BinaryExpression<'a>),
    #[from]
    EqualityExpression(EqualityExpression<'a>),
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
}

impl<'a> Expression<'a> for Expressions<'a> {
    fn line_number(&self) -> usize {
        match self {
            Expressions::BinaryExpression(binary_expression) => binary_expression.line_number(),
            Expressions::EqualityExpression(equality_expression) => {
                equality_expression.line_number()
            }
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
        }
    }
}

impl<'a> CodeGenerator<'a> for Expressions<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk<'a>,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        match self {
            Expressions::BinaryExpression(binary_expression) => {
                binary_expression.write_expression(chunk, dst_register, reserved_registers)
            }
            Expressions::EqualityExpression(equality_expression) => {
                equality_expression.write_expression(chunk, dst_register, reserved_registers)
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
        }
    }
}
