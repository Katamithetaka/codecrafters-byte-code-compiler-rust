
use std::{ fmt::{Debug, Display}, rc::Rc};

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
#[derive(Copy, Clone, PartialEq)]
pub struct Value(u64);

#[derive(Debug)]
pub enum ValueDebugType {
    Number(f64),
    Null,
    False,
    True,
    String(u32),
    Closure(u32),
    Function(u32),
    GlobalFn(u32),
    Class(u32),
    Instance(u32),

    Cell(u32),
}

const QNAN: u64     = 0x7FFC_0000_0000_0000;
const SIGN_BIT: u64 = 0x8000_0000_0000_0000;

// Non-GC types: QNAN set, sign bit clear, low bits used as tag
const TAG_NULL:    u64 = QNAN | 1;
const TAG_FALSE:   u64 = QNAN | 2;
const TAG_TRUE:    u64 = QNAN | 3;

// GC types: QNAN set, sign bit ALSO set, low 3 bits = type tag
const TAG_STRING:      u64 = SIGN_BIT | QNAN | 0;
const TAG_CLOSURE:     u64 = SIGN_BIT | QNAN | 1;
const TAG_FUNCTION:    u64 = SIGN_BIT | QNAN | 2;
const TAG_GLOBAL_FN:   u64 = SIGN_BIT | QNAN | 3;
const TAG_CLASS:       u64 = SIGN_BIT | QNAN | 4;
const TAG_INSTANCE:    u64 = SIGN_BIT | QNAN | 5;
const TAG_CELL:        u64 = SIGN_BIT | QNAN | 6;

const IDX_SHIFT: u64 = 3;
const IDX_MASK: u64    = 0x0000_0000_FFFF_FFFF;  // low 32 bits

impl Value {
    #[inline(always)]
    pub fn number(n: f64) -> Self {
        Value(n.to_bits())
    }

    #[inline(always)]
    pub fn is_number(self) -> bool {
        // if it's not a NaN, it's a number
        // also handle the case where it's a real NaN float
        (self.0 & QNAN) != QNAN
    }

    #[inline(always)]
    pub fn as_number(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[inline(always)]
    pub fn null() -> Self { Value(TAG_NULL) }

    #[inline(always)]
    pub fn bool(b: bool) -> Self {
        if b { Value(TAG_TRUE) } else { Value(TAG_FALSE) }
    }

    #[inline(always)]
    pub fn is_bool(self) -> bool {
        self.0 == TAG_TRUE || self.0 == TAG_FALSE
    }

    #[inline(always)]
    pub fn as_bool(self) -> bool {
        self.0 == TAG_TRUE
    }

    #[inline(always)]
    pub fn is_null(self) -> bool { self.0 == TAG_NULL }

    #[inline(always)]
    fn gc(tag: u64, gc: Gc) -> Self {
        Value(tag | ((gc.0 as u64) << IDX_SHIFT))
    }

    #[inline(always)]
    fn as_gc(self) -> Gc {
        Gc(((self.0 >> IDX_SHIFT) & 0xFFFF_FFFF) as u32)
    }

    pub fn string(gc: Gc) -> Self   { Self::gc(TAG_STRING, gc) }
    pub fn closure(gc: Gc) -> Self  { Self::gc(TAG_CLOSURE, gc) }
    pub fn function(gc: Gc) -> Self { Self::gc(TAG_FUNCTION, gc) }
    pub fn global_fn(gc: Gc) -> Self { Self::gc(TAG_GLOBAL_FN, gc) }
    pub fn class(gc: Gc) -> Self    { Self::gc(TAG_CLASS, gc) }
    pub fn instance(gc: Gc) -> Self { Self::gc(TAG_INSTANCE, gc) }
    pub fn cell(gc: Gc) -> Self     { Self::gc(TAG_CELL, gc) }

    #[inline(always)]
    pub fn tag(self) -> u64 {
        self.0 & (SIGN_BIT | QNAN | 0x7)
    }

    pub fn is_string(self) -> bool   { self.tag() == TAG_STRING }
    pub fn is_closure(self) -> bool  { self.tag() == TAG_CLOSURE }
    pub fn is_function(self) -> bool { self.tag() == TAG_FUNCTION }
    pub fn is_global_function(self) -> bool { self.tag() == TAG_GLOBAL_FN }
    pub fn is_class(self) -> bool    { self.tag() == TAG_CLASS }
    pub fn is_instance(self) -> bool { self.tag() == TAG_INSTANCE }
    pub fn is_cell(self) -> bool     { self.tag() == TAG_CELL }

    pub fn unwrap_gc(self) -> Gc { self.as_gc() }

    pub fn as_debug_type(self) -> ValueDebugType {
        if self.is_number() {
            return ValueDebugType::Number(self.as_number());
        };

        if self.is_null() {
            return ValueDebugType::Null;
        };

        if self.is_bool() {
            return match self.as_bool() {
                true => ValueDebugType::True,
                false => ValueDebugType::False,
            }
        };


        let gc = self.as_gc();
        match self.tag() {
            _ if self.is_string() => ValueDebugType::String(gc.0),
            _ if self.is_closure() => ValueDebugType::Closure(gc.0),
            _ if self.is_function() => ValueDebugType::Function(gc.0),
            _ if self.is_class() => ValueDebugType::Class(gc.0),
            _ if self.is_instance() => ValueDebugType::Instance(gc.0),
            _ if self.is_cell() => ValueDebugType::Cell(gc.0),
            _ if self.is_global_function() => ValueDebugType::GlobalFn(gc.0),

            _ => unreachable!()
        }



    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value").field(&self.as_debug_type()).finish()
    }
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
        Value::null()
    }
}
