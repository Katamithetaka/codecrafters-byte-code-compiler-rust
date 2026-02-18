use std::{fmt::Display, rc::Rc};

use crate::{prelude::Chunk, value::Value};



/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug)]
pub struct FunctionInner<T> {
    /// The name of the function.
    pub name: String,

    /// The number of arguments the function takes.
    pub arguments_count: u8,

    pub chunk: Rc<Chunk<T>>
}

/// Represents a user-defined function in the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// The internal details of the function.
    inner: Rc<FunctionInner<String>>
}

#[derive(Debug, Clone)]
pub struct Closure<T> {
    pub function: Function,           // function metadata (arity, code begin)
    pub chunk: Rc<Chunk<T>>,         // bytecode for this function
    pub upvalues: Vec<Value<T>>, // captured variables
}

impl<T> PartialEq for Closure<T> {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
    }
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

impl<T> PartialEq for FunctionInner<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments_count == other.arguments_count
    }
}

impl Function {
    pub fn new(name: String, arguments_count: u8, chunk: Rc<Chunk<String>>) -> Self {
        Self {
            inner: Rc::new(FunctionInner {
                name,
                arguments_count,
                chunk
            }),
        }
    }

    pub fn name(&self) -> &str {
        return &self.inner.name
    }



    pub fn arguments_count(&self) -> u8 {
        return self.inner.arguments_count
    }

    pub fn chunk(&self) -> Rc<Chunk<String>> {
        return self.inner.chunk.clone()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.inner.name)
    }
}
