use std::num::TryFromIntError;

use crate::{
    compiler::{
        instructions::{Instructions, disassemble_instruction},
        varint::Varint,
    },
    expressions::Value,
};

#[derive(Debug)]
pub struct Chunk<'a> {
    pub code: Vec<u8>,
    pub constants: Vec<Value<&'a str>>,
    pub lines: Vec<(i32, usize)>,
}

impl<'a> Chunk<'a> {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        };
    }
    pub fn disassemble(&self, name: &str) {
        eprintln!("== {} ==", name);
        let mut i = 0;
        let mut previous = i;
        while i < self.code.len() {
            let tmp = i;
            i = disassemble_instruction(self, i, previous);
            previous = tmp;
        }
    }

    pub fn get_constant(&mut self, value: &Value<&str>) -> Option<Varint> {
        self.constants.iter().position(|e| e == value).map(|v| {
            return Varint(v as u32);
        })
    }

    pub fn add_constant(&mut self, value: Value<&'a str>) -> Varint {
        self.constants.push(value);
        return Varint((self.constants.len() - 1) as u32);
    }

    pub fn get_or_write_constant(&mut self, value: Value<&'a str>, line: i32) -> Varint {
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

        self.code.push(byte);
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

    pub fn write_jump_if_false_placeholder(&mut self, register_index: u8, line: i32) -> usize {
        self.write_instruction(Instructions::JumpIfFalse, line);
        self.write(register_index as u8, line);
        let return_val = self.code.len();
        self.write(0xFF as u8, line);
        self.write(0xFF as u8, line);
        return_val
    }

    pub fn update_jump_if_false(&mut self, index: usize) -> Result<(), TryFromIntError> {
        let current_offset: u16 = self.code.len().try_into()?;
        let values: [u8; 2] = current_offset.to_be_bytes(); // IT IS NOW DECIDED THAT WE USE BIG ENDIAN LMAO
        self.code[index] = values[0];
        self.code[index + 1] = values[1];
        Ok(())
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

    pub fn write_declare_local(&mut self, value_register: u8, line: i32) {
        self.write_instruction(Instructions::DefineLocal, line);
        self.write(value_register as u8, line);
    }

    pub fn write_get_local(&mut self, output_register: u8, depth: u8, index: u8, line: i32) {
        self.write_instruction(Instructions::GetLocal, line);
        self.write(output_register as u8, line);
        self.write(depth as u8, line);
        self.write(index as u8, line);
    }

    pub fn write_set_local(&mut self, input_register: u8, depth: u8, index: u8, line: i32) {
        self.write_instruction(Instructions::SetLocal, line);
        self.write(input_register as u8, line);
        self.write(depth as u8, line);
        self.write(index as u8, line);
    }

    pub fn write_set_global(&mut self, ident: Varint, value_register: u8, line: i32) -> usize {
        self.write_instruction(Instructions::SetGlobal, line);
        self.write(value_register as u8, line);
        ident.write_bytes(self, line)
    }

    pub fn write_get_global(&mut self, ident: Varint, dst_register: u8, line: i32) -> usize {
        self.write_instruction(Instructions::GetGlobal, line);
        self.write(dst_register as u8, line);
        ident.write_bytes(self, line)
    }

    pub fn write_stack_push(&mut self, line: i32) {
        self.write_instruction(Instructions::PushStack, line);
    }

    pub fn write_stack_pop(&mut self, line: i32) {
        self.write_instruction(Instructions::PopStack, line);
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
}

impl Default for Chunk<'_> {
    fn default() -> Self {
        Self::new()
    }
}
