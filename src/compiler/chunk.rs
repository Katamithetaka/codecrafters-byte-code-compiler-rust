use std::io::Write;

use crate::compiler::{
    instructions::{Instructions, disassemble_instruction},
    value::{Value, ValueArray},
    varint::Varint,
};

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub value_array: ValueArray,
    pub lines: Vec<(i32, usize)>,
}

impl Chunk {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            value_array: vec![],
            lines: vec![],
        };
    }
    pub fn disassemble(&self, name: &str) {
        eprintln!("== {} ==", name);
        let mut i = 0;
        while (i < self.code.len()) {
            i = disassemble_instruction(self, i)
        }
    }

    pub fn get_constant(&mut self, value: &Value) -> Option<Varint> {
        self.value_array.iter().position(|e| e == value).map(|v| {
            return Varint(v as u32);
        })
    }

    pub fn add_constant(&mut self, value: Value) -> Varint {
        self.value_array.push(value);
        return Varint((self.value_array.len() - 1) as u32);
    }

    pub fn get_or_write_constant(&mut self, value: Value, line: i32) -> Varint {
        match self.get_constant(&value) {
            Some(constant) => constant,
            None => {
                let constant = self.add_constant(value);
                self.write_constant(constant, line as i32);
                constant
            }
        }
    }

    pub fn write(&mut self, byte: u8, line: i32) {
        let line_v = self.lines.last_mut();
        match line_v {
            Some(a) if a.0 == line => a.1 += 1,
            _ => self.lines.push((line, 1)),
        }

        self.write_all(&[byte]).unwrap();
    }

    pub fn write_instruction(&mut self, byte: Instructions, line: i32) {
        return self.write(byte as u8, line);
    }

    pub fn write_constant(&mut self, constant: Varint, line: i32) -> usize {
        self.write_instruction(Instructions::Constant, line);
        constant.write_bytes(self, line)
    }

    pub fn write_binary(&mut self, byte: Instructions, r0: u8, r1: u8, dst: u8, line: i32) {
        self.write(byte as u8, line);
        self.write(r0 as u8, line);
        self.write(r1 as u8, line);
        self.write(dst as u8, line);
    }

    pub fn write_load(&mut self, register_index: u8, constant: Varint, line: i32) -> usize {
        self.write_instruction(Instructions::Load, line);
        self.write(register_index as u8, line);
        constant.write_bytes(self, line)
    }

    pub fn write_print(&mut self, register_index: u8, line: i32) {
        self.write_instruction(Instructions::Print, line);
        self.write(register_index as u8, line);
    }

    pub fn write_declare_global(&mut self, ident: Varint, value_register: u8, line: i32) -> usize {
        self.write_instruction(Instructions::DefineGlobal, line);
        self.write(value_register as u8, line);
        ident.write_bytes(self, line)
    }

    pub fn write_get_global(&mut self, ident: Varint, dst_register: u8, line: i32) -> usize {
        self.write_instruction(Instructions::GetGlobal, line);
        self.write(dst_register as u8, line);
        ident.write_bytes(self, line)
    }

    pub fn write_unary(
        &mut self,
        byte: Instructions,
        register_index: u8,
        dst_register_index: u8,
        line: i32,
    ) {
        self.write(byte as u8, line);
        self.write(register_index as u8, line);
        self.write(dst_register_index as u8, line);
    }

    pub fn get_line(&self, offset: usize) -> i32 {
        let mut i = 0;
        let mut line_index = 0;
        while (i < offset) {
            i += self.lines[line_index].1;
            if i >= offset {
                break;
            }
            line_index += 1;
        }
        return self.lines[line_index].0;
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for Chunk {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.code.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
