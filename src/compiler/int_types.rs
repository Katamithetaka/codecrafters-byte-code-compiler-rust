#![allow(non_camel_case_types)]


use crate::{compiler::vm::Vm, prelude::Chunk};

pub type varint_type = u32;
pub type line_type = usize;
pub type stack_index_type = u8;
pub type stack_pointer_type = u16;
pub type register_index_type = u8;
pub type global_index_type = u8;
pub type instruction_length_type = u16;

pub trait ChunkRead {
    fn read(chunk: &Chunk,  offset: &mut usize) -> Self;
}

pub trait VmRead {
    fn read_bytes(vm: &mut Vm) -> Self;
}

impl<T: ChunkRead> VmRead for T {
    fn read_bytes(vm: &mut Vm) -> Self {
        T::read(vm.core.current_chunk, &mut vm.core.ip)
    }
}

impl ChunkRead for u32 {
    fn read(chunk: &Chunk,  offset: &mut usize) -> Self {
        let v: u32 = u32::from_be_bytes([chunk.code[*offset], chunk.code[*offset + 1], chunk.code[*offset + 2], chunk.code[*offset + 3]]);
        *offset += 4;
        return v;
    }
}

impl ChunkRead for u16 {
    fn read(chunk: &Chunk,  offset: &mut usize) -> Self {
        let v: u16 = u16::from_be_bytes([chunk.code[*offset], chunk.code[*offset + 1]]);
        *offset += 2;
        return v;
    }
}

impl ChunkRead for u8 {
    fn read(chunk: &Chunk,  offset: &mut usize) -> Self {
        let v: u8 = u8::from_be_bytes([chunk.code[*offset]]);
        *offset += 1;
        return v;
    }
}
