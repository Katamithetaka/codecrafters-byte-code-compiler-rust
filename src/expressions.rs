use std::fmt::{Debug, Display};

use crate::expressions::binary_expression::BinaryOp;

pub mod binary_expression;
pub mod equality_expression;
pub mod group;
pub mod literal;
pub mod relation_expression;
pub mod unary_expression;

#[derive(PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Null,
    Boolean(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "{}", s),
            Value::Null => f.write_str("nil"),
            Value::Boolean(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(v) => *v,
            _ => true,
        }
    }

    pub fn binary_op_compatible(&self, other: &Self, op: BinaryOp) -> Option<EvaluateErrorDetails> {
        match (self, other) {
            (Value::Number(_), Value::Number(_)) => None,

            (Value::String(_), Value::String(_)) if (op == BinaryOp::Plus) => None,
            (Value::Number(_), _) | (_, Value::Number(_)) => {
                Some(EvaluateErrorDetails::UnmatchedTypes)
            }
            (Value::String(_), _) | (_, Value::String(_)) if op == BinaryOp::Plus => {
                Some(EvaluateErrorDetails::UnmatchedTypes)
            }
            _ => Some(EvaluateErrorDetails::BinaryNumberOp),
        }
    }

    pub fn add(&self, right: &Self) -> Self {
        match (self, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(format!("{a}{b}")),
            _ => panic!("Tried to apply add on incompatible values!"),
        }
    }

    pub fn sub(&self, right: &Self) -> Self {
        match (self, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => panic!("Tried to apply sub on incompatible values!"),
        }
    }

    pub fn div(&self, right: &Self) -> Self {
        match (self, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => panic!("Tried to apply sub on incompatible values!"),
        }
    }

    pub fn mult(&self, right: &Self) -> Self {
        match (self, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => panic!("Tried to apply sub on incompatible values!"),
        }
    }
}

pub enum EvaluateOutcomeDetails {
    Value(Option<Value>),
    Return(Option<Value>),
}

impl Display for EvaluateOutcomeDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluateOutcomeDetails::Value(value) => {
                write!(f, "{}", value.as_ref().unwrap_or(&Value::Null))
            }
            EvaluateOutcomeDetails::Return(value) => todo!(),
        }
    }
}

pub struct EvaluateOutcome {
    details: EvaluateOutcomeDetails,
    line: usize,
}

impl Display for EvaluateOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EvaluateErrorDetails {
    #[error("Unexpected return statement")]
    UnexpectedReturn,
    #[error("Evaluate error: {0}")]
    Error(String),
    #[error("Expected value from expressio, got None")]
    ExpectedValue,
    #[error("Operands must be numbers.")]
    BinaryNumberOp,
    #[error("Operand must be number.")]
    UnaryNumberOp,
    #[error("Operands must be two numbers or two strings. ")]
    UnmatchedTypes,
}

#[derive(thiserror::Error, Debug)]
pub struct EvaluateError {
    error: EvaluateErrorDetails,
    line: usize,
}

impl Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.error)
    }
}

pub type Result = std::result::Result<EvaluateOutcome, EvaluateError>;

pub fn expect_ok(res: Result) -> std::result::Result<Option<Value>, EvaluateError> {
    match res {
        Ok(v) => match v {
            EvaluateOutcome {
                details: EvaluateOutcomeDetails::Value(v),
                line,
            } => Ok(v),
            EvaluateOutcome {
                details: EvaluateOutcomeDetails::Return(_),
                line,
            } => Err(EvaluateError {
                error: EvaluateErrorDetails::UnexpectedReturn,
                line,
            }),
        },
        Err(v) => Err(v),
    }
}

pub trait Expression: Display + Debug {
    fn line_number(&self) -> usize;

    fn ok(&self, v: Option<Value>) -> Result {
        return Ok(EvaluateOutcome {
            details: EvaluateOutcomeDetails::Value(v),
            line: self.line_number(),
        });
    }

    fn ret(&self, v: Option<Value>) -> Result {
        return Ok(EvaluateOutcome {
            details: EvaluateOutcomeDetails::Return(v),
            line: self.line_number(),
        });
    }

    fn err(&self, v: EvaluateErrorDetails) -> Result {
        return Err(EvaluateError {
            error: v,
            line: self.line_number(),
        });
    }

    fn evaluate(&mut self) -> Result;
}
