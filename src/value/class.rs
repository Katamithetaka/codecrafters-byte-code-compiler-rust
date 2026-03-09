use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::value::{Closure, callable::Callable};

/// Represents the internal details of a function, including its name, starting position, and argument count.
#[derive(Clone, Debug)]
pub struct ClassInner {
    /// The name of the function.
    pub name: String,
    pub base_class: Option<Class>,
    pub constructor: Option<Callable>,
    pub methods: Vec<Callable>
}

/// Represents a user-defined function in the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    /// The internal details of the function.
    inner: Rc<RefCell<ClassInner>>,
}

impl PartialEq for ClassInner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.borrow().name)
    }
}

impl Class {

    pub fn new(name: String) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ClassInner {
                name,
                base_class: None,
                methods: vec![],
                constructor: None
            })),
        }
    }

    pub fn add_method(&mut self, method: Callable) {
        self.inner.borrow_mut().methods.push(method);
    }

    pub fn methods(&self) -> Vec<Callable> {
        self.inner.borrow().methods.clone()
    }

    pub fn has_method(&self, name: String) -> bool {
        self.inner.borrow().methods.iter().find(|c| c.name() == name).is_some()
    }

    pub fn set_base_class(&mut self, class: Class) {
        self.inner.borrow_mut().base_class = Some(class);
    }

    pub fn base_class(&self) -> Option<Class> {
        self.inner.borrow().base_class.clone()
    }

    pub fn name(&self) -> String {
        self.inner.borrow().name.clone()
    }

    pub fn constructor(&self) -> Option<Callable> {
        return self.inner.borrow().constructor.clone();
    }

    pub fn set_constructor(&mut self, callabl: Callable) {
        self.inner.borrow_mut().constructor  = Some(callabl);

    }
}
