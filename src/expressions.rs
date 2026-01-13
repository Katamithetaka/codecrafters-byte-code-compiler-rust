use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};

use crate::{
    compiler::{CodeGenerator, chunk::Chunk},
    expressions::{
        assignment_expression::AssignmentExpression, binary_expression::BinaryExpression, call_expression::CallExpression, equality_expression::EqualityExpression, group::Group, identifier::Identifier, literal::Literal, logical_expression::LogicalExpression, relation_expression::RelationalExpression, unary_expression::UnaryExpression
    },
};

pub mod assignment_expression;
pub mod binary_expression;
pub mod equality_expression;
pub mod group;
pub mod identifier;
pub mod literal;
pub mod logical_expression;
pub mod relation_expression;
pub mod unary_expression;
pub mod call_expression;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInner {
    pub name: String,
    pub begin: u16,
    pub arguments_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    inner: Rc<FunctionInner>
}

#[derive(Clone, Debug)] 
pub struct GlobalFunction {
    pub callable: Rc<fn(Vec<Value<String>>) -> Value<String>>,
    pub name: &'static str,
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


#[derive(Clone, PartialEq, Debug)]
pub enum Value<S> {
    Number(f64),
    String(S),
    Null,
    Boolean(bool),
    Function(Function),
    GlobalFunction(GlobalFunction),
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
    #[error("Operand must be function.")]
    ExpectedFunction,
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
    #[error("Invalid number of arguments when calling function!")]
    InvalidArgCount,
    #[error("Call stack wasn't pushed before calling a function!")]
    CallStackEmpty,
    #[error("Invalid return statement")]
    InvalidReturnStatement,
    #[error("Jump statement didn't fit in the boundaries of a u16")]
    CodeTooLong,
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
