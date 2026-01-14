use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};

use crate::{
    compiler::{CodeGenerator, chunk::Chunk},
    expressions::{
        assignment_expression::AssignmentExpression, binary_expression::BinaryExpression, call_expression::CallExpression, equality_expression::EqualityExpression, group::Group, identifier::Identifier, literal::Literal, logical_expression::LogicalExpression, relation_expression::RelationalExpression, unary_expression::UnaryExpression
    },
};

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

/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInner {
    /// The name of the function.
    pub name: String,
    /// The starting position of the function in the bytecode.
    pub begin: u16,
    /// The number of arguments the function takes.
    pub arguments_count: u8,
}

/// Represents a user-defined function in the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// The internal details of the function.
    inner: Rc<FunctionInner>
}

/// Represents a global function that can be called from anywhere in the program.
#[derive(Clone, Debug)] 
pub struct GlobalFunction {
    /// A reference-counted function pointer to the callable implementation.
    pub callable: Rc<fn(Vec<Value<String>>) -> Value<String>>,
    /// The name of the global function.
    pub name: &'static str,
    /// The number of arguments the global function takes.
    pub arguments_count: u8,
}

impl Display for GlobalFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name) 
    }
}

impl PartialEq for GlobalFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Function {
    pub fn new(name: String, begin: u16, arguments_count: u8) -> Self {
        Self {
            inner: Rc::new(FunctionInner {
                name,
                begin,
                arguments_count,
            }),
        }
    }
    
    pub fn name(&self) -> &str {
        return &self.inner.name
    }
    
    pub fn begin(&self) -> u16 {
        return self.inner.begin
    }
    
    pub fn arguments_count(&self) -> u8 {
        return self.inner.arguments_count
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.inner.name) 
    }
}


/// Represents a value in the interpreter, which can be one of several types.
#[derive(Clone, PartialEq, Debug)]
pub enum Value<S> {
    /// A numeric value.
    Number(f64),
    /// A string value.
    String(S),
    /// A null value.
    Null,
    /// A boolean value.
    Boolean(bool),
    /// A user-defined function.
    Function(Function),
    /// A global function.
    GlobalFunction(GlobalFunction),
    /// A reference-counted, mutable value.
    Cell(Rc<RefCell<Value<S>>>)
}

impl<S: Display> Display for Value<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "{}", s),
            Value::Null => f.write_str("nil"),
            Value::Boolean(s) => write!(f, "{}", s),
            Value::Function(s) => write!(f, "{}", s),
            Value::GlobalFunction(s) => write!(f, "{}", s),
            Value::Cell(s) => write!(f, "{}", s.borrow()), 
        }
    }
}

impl<S> Default for Value<S> {
    fn default() -> Self {
        Value::Null
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
            Value::Function(b) => Value::Function(b),
            Value::GlobalFunction(b) => Value::GlobalFunction(b),
            Value::Cell(inner) => {
                let new_inner = Rc::new(RefCell::new((*inner.borrow()).clone().into()));
                Value::Cell(new_inner)
            },  
        }
    }
}

impl<S: ToString> Value<S> {
    pub fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            Value::Cell(cell) => cell.borrow().is_null(), // recursive unwrap
            _ => false,
        }
    }
    
    /// Recursively unwrap a number, even if it's inside a Cell
    pub fn as_number(&self) -> Result<f64, EvaluateErrorDetails> {
        match self {
            Value::Number(a) => Ok(*a),
            Value::Cell(cell) => cell.borrow().as_number(), // recursive unwrap
            _ => Err(EvaluateErrorDetails::ExpectedNumber),
        }
    }

    /// Recursively unwrap a string, even if it's inside a Cell
    pub fn as_string(&self) -> Result<String, EvaluateErrorDetails> {
        match self {
            Value::String(a) => {
                Ok(a.to_string())
            }
            Value::Cell(cell) => {
                let borrow = cell.borrow();      // Ref<Value<S>>
                // map Ref<Value<S>> -> Ref<S>
                borrow.as_string()
            }
            _ => Err(EvaluateErrorDetails::ExpectedString),
        }
    }

    /// Recursively unwrap an identifier
    pub fn as_ident(&self) -> Result<String, EvaluateErrorDetails> {
        self.as_string().map_err(|_| EvaluateErrorDetails::InvalidIdentifierType)
    }

    /// Recursively unwrap for binary number operations
    pub fn as_binary_number_op(&self) -> Result<f64, EvaluateErrorDetails> {
        self.as_number().map_err(|_| EvaluateErrorDetails::BinaryNumberOp)
    }

    /// Recursively unwrap a function
    pub fn as_function(&self) -> Result<Function, EvaluateErrorDetails> {
        match self {
            Value::Function(f) => Ok(f.clone()),
            Value::Cell(cell) => cell.borrow().as_function(), // recursive unwrap
            _ => Err(EvaluateErrorDetails::ExpectedFunction),
        }
    }

    /// Recursively unwrap a global function
    pub fn as_global_function(&self) -> Result<GlobalFunction, EvaluateErrorDetails> {
        match self {
            Value::GlobalFunction(f) => Ok(f.clone()),
            Value::Cell(cell) => cell.borrow().as_global_function(), // recursive unwrap
            _ => Err(EvaluateErrorDetails::ExpectedFunction),
        }
    }
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
    CallExpressionn(CallExpression<'a>),
}

impl<'a> Expression<'a> for Expressions<'a> {
    fn line_number(&self) -> usize {
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
            Expressions::CallExpressionn(call_expression) =>  {
                call_expression.line_number()
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
            Expressions::CallExpressionn(call_expression) => {
                call_expression.write_expression(chunk, dst_register, reserved_registers)
            }
        }
    }
}
