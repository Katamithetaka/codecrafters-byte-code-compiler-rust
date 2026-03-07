use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::value::{Value, class::Class};

/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug)]
pub struct ClassInstanceInner {
    pub class: Class,
    pub fields: RefCell<HashMap<String, Value<String>>>
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
                fields: RefCell::new(HashMap::new()),
            }),
        }
    }

    pub fn get_field(&self, field_name: &str) -> Value<String> {
        match self.inner.fields.borrow().get(field_name) {
            Some(v) => v.clone(),
            None => Value::Null,
        }
    }

    pub fn set_field(&mut self, field_name: String, value: Value<String>) {
        let value = value.into_cell();

        self.inner.fields.borrow_mut().entry(field_name).and_modify(|c| *c = value.clone()).or_insert(value.clone());
    }
}
