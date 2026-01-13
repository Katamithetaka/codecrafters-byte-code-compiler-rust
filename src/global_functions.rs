use std::rc::Rc;

use crate::{compiler::chunk::Chunk, expressions::{GlobalFunction, Value}};

pub mod clock;


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

pub fn register_global_functions(chunk: &mut Chunk) {
    
    let functions = global_mods!(clock);

    for func in functions {
        let name = chunk.add_constant(Value::String(func.name));
        let constant = chunk.add_constant(Value::GlobalFunction(func));
        chunk.write_load(0, constant, 0);
        chunk.write_declare_global(name, 0, 0);
    }
}