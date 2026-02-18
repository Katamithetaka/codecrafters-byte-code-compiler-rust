use std::{fmt::Display, rc::Rc};

use crate::{compiler::{compiler::Compiler, varint::Varint}, value::class::{self, Class}};

/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug)]
pub struct ClassInstanceInner {
    pub class: Class
}

/// Represents a user-defined function in the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassInstance {
    /// The internal details of the function.
    inner: Rc<ClassInstanceInner>
}

impl PartialEq for ClassInstanceInner {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.inner.class.name())
    }
}

impl ClassInstance {

    pub fn new(class: Class) -> Self {
        Self {
            inner: Rc::new(ClassInstanceInner {
                class,
            }),
        }
    }
}
