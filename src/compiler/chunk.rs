
use crate::{
    compiler::{
        garbage_collector::Heap, instructions::disassemble_instruction, int_types::line_type, varint::Varint
    },
    expressions::Value,
};

#[derive(Debug,  Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<(line_type, usize)>,
}





impl Chunk {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        };
    }

    pub fn add_constant(&mut self, value: Value) -> Varint {
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

    pub fn disassemble(self: &Chunk, heap: &Heap, name: &str) {
        eprintln!("== {} ==", name);
        let mut i = 0;
        let mut previous = i;
        while i < self.code.len() {
            let tmp = i;
            i = disassemble_instruction(heap, &self, i, previous);
            previous = tmp;
        }
    }
}

impl Chunk {
    pub fn get_constant(&mut self, value: &Value) -> Option<Varint> {
        self.constants.iter().position(|e| e == value).map(|v| {
            return Varint(v as u32);
        })
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
