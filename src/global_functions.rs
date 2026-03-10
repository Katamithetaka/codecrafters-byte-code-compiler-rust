/// This module defines global functions that can be registered and used within the interpreter.
/// It includes utilities for defining and registering global functions.

use std::rc::Rc;

use crate::{compiler::{compiler::{Compiler, ResolvedVar}, garbage_collector::HeapObject}, value::{GlobalFunction, Value}};

pub mod clock;

/// The `prelude` module re-exports commonly used items from this module for easier access.
///
/// # Examples
/// ```
/// use interpreter::global_functions::prelude::*;
/// ```
pub mod prelude {
    pub use super::register_global_functions;
}

macro_rules! global_mod {
    ($mod: ident) => {
        GlobalFunction {
            callable: Rc::new($mod::execute),
            name: $mod::NAME,
            arguments_count: $mod::NUM_ARGUMENTS,
        }
    };
}

macro_rules! global_mods {
    ( $( $mod:ident ),* ) => {
        {
            let mut v = Vec::new();
            $(
                v.push(global_mod!($mod));
            )*
            v
        }
    };
}

/// Registers all global functions into the given chunk.
///
/// This function adds predefined global functions to the chunk, making them available
/// for use in the interpreter. Each function is added as a global constant and can be
/// accessed by its name.
///
/// # Arguments
///
/// * `chunk` - The chunk where the global functions will be registered.
///
/// # Examples
/// ```
/// use interpreter::global_functions::register_global_functions;
/// use interpreter::compiler::chunk::Chunk;
///
/// let mut chunk = Chunk::new();
/// register_global_functions(&mut chunk);
/// ```
pub fn register_global_functions(chunk: &mut Compiler) {

    let functions = global_mods!(clock);

    for func in functions {

        chunk.add_global(func.name.to_string());
        match chunk.resolve_variable(func.name).unwrap() {

            ResolvedVar::Global(varint) => {
                let v = chunk.heap().borrow_mut().alloc(HeapObject::GlobalFunction(func));
                let constant = chunk.add_constant(Value::GlobalFunction(v));
                chunk.write_load(0, constant, 0);
                chunk.write_declare_global(varint, 0, 0);
            },
            _ => unreachable!(),
        }

    }
}
