use std::{fmt::Display, rc::Rc};

use crate::{
    compiler::{
        instructions::disassemble_instruction, int_types::line_type, varint::Varint
    },
    expressions::Value,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Chunk<S> {
    pub code: Vec<u8>,
    pub constants: Vec<Value<S>>,
    pub lines: Vec<(line_type, usize)>,
}





impl<S> Chunk<S> {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        };
    }

    pub fn add_constant(&mut self, value: Value<S>) -> Varint {
        self.constants.push(value);
        return Varint((self.constants.len() - 1) as u32);
    }

    pub fn write(&mut self, byte: u8, line: line_type) {
        let line_v = self.lines.last_mut();
        match line_v {
            Some(a) if a.0 == line => a.1 += 1,
            _ => self.lines.push((line, 1)),
        }

        self.code.push(byte);
    }


    pub fn get_line(&self, offset: usize) -> line_type {
        let mut i = 0;
        let mut line_index = 0;
        while i < offset {
            i += self.lines[line_index].1;
            if i >= offset {
                break;
            }
            if line_index + 1 >= self.lines.len() {
                break;
            }
            line_index += 1;
        }
        return self.lines[line_index].0;
    }

    pub fn disassemble(self: Rc<Chunk<S>>, name: &str) where S: Display {
        eprintln!("== {} ==", name);
        let mut i = 0;
        let mut previous = i;
        while i < self.code.len() {
            let tmp = i;
            i = disassemble_instruction(Rc::clone(&self), i, previous);
            previous = tmp;
        }
    }
}

impl<S: PartialEq> Chunk<S> {
    pub fn get_constant(&mut self, value: &Value<S>) -> Option<Varint> {
        self.constants.iter().position(|e| e == value).map(|v| {
            return Varint(v as u32);
        })
    }
}

impl<S> Default for Chunk<S> {
    fn default() -> Self {
        Self::new()
    }
}


impl Into<Chunk<String>> for Chunk<&str> {
    fn into(self) -> Chunk<String> {
        let Chunk { code, constants, lines } = self;
        let constants = constants.into_iter().map(|v| v.into()).collect::<Vec<Value<String>>>();
        Chunk {
            code, constants, lines
        }
    }
}
