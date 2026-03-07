use std::{fmt::Display, rc::Rc};

use crate::{value::{Closure}};

/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug)]
pub struct ClassInner {
    /// The name of the function.
    pub name: String,

    pub constructor: Option<Closure<String>>
}

/// Represents a user-defined function in the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    /// The internal details of the function.
    inner: Rc<ClassInner>
}

impl PartialEq for ClassInner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.name)
    }
}

impl Class {

    pub fn new(name: String) -> Self {
        Self {
            inner: Rc::new(ClassInner {
                name,
                constructor: None
            }),
        }
    }

    pub fn name(&self) -> String {
        self.inner.name.clone()
    }
}
