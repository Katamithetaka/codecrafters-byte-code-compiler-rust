use std::{fmt::Display, rc::Rc};

use crate::{prelude::Chunk, value::{Value, class_instance::ClassInstance}};




/// Represents a user-defined function in the interpreter.
#[derive(Clone, Debug)]
pub struct Function<T> {
    /// The name of the function.
    pub name: String,

    /// The number of arguments the function takes.
    pub arguments_count: u8,

    pub chunk: Rc<Chunk<T>>
}

impl<T> PartialEq for Function<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments_count == other.arguments_count && self.chunk.code == other.chunk.code
    }
}

#[derive(Debug, Clone)]
pub struct Closure<T> {
    pub function: Function<T>,      // function metadata (arity, code begin)
    pub upvalues: Vec<Value<T>>,    // captured variables
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
    pub arguments_count: Option<u8>,
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


impl Function<String> {
    pub fn new(name: String, arguments_count: u8, chunk: Rc<Chunk<String>>) -> Self {
        Self {
            name,
            arguments_count,
            chunk,
        }
    }


}

impl<T> Display for Function<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}


pub enum Callable {
    LoxFunction(Closure<String>),
    GlobalFunction(GlobalFunction),
    BindedLoxFunction(ClassInstance, Closure<String>)
}

impl Callable {
    pub fn bind(self, instance: ClassInstance) -> Callable {

        match self {
            Callable::LoxFunction(closure) => Callable::BindedLoxFunction(instance, closure),
            Callable::GlobalFunction(_) => panic!("Cannot bind a global function"),
            Callable::BindedLoxFunction(_, closure) => Callable::BindedLoxFunction(instance, closure),
        }
    }
}
