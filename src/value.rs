use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::prelude::EvaluateErrorDetails;


/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInner {
    /// The name of the function.
    pub name: String,

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
    pub fn new(name: String, arguments_count: u8) -> Self {
        Self {
            inner: Rc::new(FunctionInner {
                name,
                arguments_count,
            }),
        }
    }

    pub fn name(&self) -> &str {
        return &self.inner.name
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
