
use std::{ fmt::Display, rc::Rc};

use crate::{compiler::garbage_collector::{Gc},};
pub use crate::{prelude::EvaluateErrorDetails};

/// Represents a global function that can be called from anywhere in the program.
#[derive(Clone, Debug)]
pub struct GlobalFunction {
    /// A reference-counted function pointer to the callable implementation.
    pub callable: Rc<fn(Vec<Value>) -> Value>,
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


/// Represents a value in the interpreter, which can be one of several types.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Value {
    Number(f64),
    String(Gc),
    Null,
    Boolean(bool),
    Function(Gc),
    GlobalFunction(Gc),
    Closure(Gc),
    Class(Gc),
    Instance(Gc),
    Cell(Gc)
}

// impl Display for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Value::Number(s) => write!(f, "{}", s),
//             Value::String(s) => write!(f, "{}", s),
//             Value::Null => f.write_str("nil"),
//             Value::Boolean(s) => write!(f, "{}", s),
//             Value::Function(s) => write!(f, "{}", s),
//             Value::GlobalFunction(s) => write!(f, "{}", s),
//             Value::Cell(s) => write!(f, "{}", s.borrow()),
//             Value::Closure(closure) => {
//                 match closure {
//                     Callable::LoxFunction(closure) => write!(f, "{}", closure.function.borrow()),
//                     Callable::BindedLoxFunction(_, closure) => write!(f, "{}", closure.function.borrow()),
//                 }
//             },
//             Value::Class(class) => write!(f, "{}", class),
//             Value::Instance(class) => write!(f, "{}", class),

//         }
//     }
// }

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}
