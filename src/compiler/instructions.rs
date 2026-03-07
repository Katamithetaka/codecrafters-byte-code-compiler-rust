use std::fmt::{Display};
use std::rc::Rc;

use crate::compiler::{chunk::Chunk, int_types::register_index_type, varint::Varint};
use crate::compiler::int_types::{ChunkRead, instruction_length_type, stack_index_type};
#[repr(u8)]
#[derive(strum::FromRepr, Clone, Copy)]
pub enum Instructions {
    Return = 0,
    Constant = 1,
    Load = 2,
    Negate = 3,
    Bang = 4,
    Add = 5,
    Sub = 6,
    Div = 7,
    Mul = 8,
    Lt = 9,
    Gt = 10,
    LtEq = 11,
    GtEq = 12,
    Eq = 13,
    Neq = 14,
    Print = 15,
    DefineGlobal = 16,
    GetGlobal = 17,
    SetGlobal = 18,
    PushStack = 19,
    PopStack = 20,
    DefineLocal = 21,
    GetLocal = 22,
    SetLocal = 23,
    JumpIfFalse = 24,
    Jump = 25,
    FunctionCall = 26,
    FunctionReturn = 27,
    DebugBreak = 28,
    Closure = 29,
    GetUpvalue = 30,
    SetUpvalue = 31,
    GetField = 32,
    SetField = 33,
}

pub fn simple_instruction(name: &str, offset: usize) -> usize {
    eprintln!("{name:15}");
    return offset + 1;
}



pub fn constant_instruction<T: Display>(name: &str, chunk: &Chunk<T>, offset: usize) -> usize {
    let (constant, o) = Varint::read_bytes(chunk, offset + 1);
    eprint!("{name:15} c{constant} ");
    eprintln!("{}", &chunk.constants[constant as usize]);

    return offset + o + 1;
}

pub fn constant_register_instruction<T: Display>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let register = register_index_type::read(Rc::clone(&chunk), &mut offset);
    let (constant, o) = Varint::read_bytes(&chunk, offset);
    eprint!("{name:15} r{} c{} ", register, constant);
    eprintln!("{}", &chunk.constants[constant as usize]);

    return offset + o;
}

pub fn constant_set_register_instruction<T: Display>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let value_register = register_index_type::read(Rc::clone(&chunk), &mut offset);
    let dst_register = register_index_type::read(Rc::clone(&chunk), &mut offset);

    let (constant, o) = Varint::read_bytes(&chunk, offset);
    eprint!("{name:15} r{}, r{} c{} ", value_register, dst_register, constant);
    eprintln!("{}", &chunk.constants[constant as usize]);

    return offset + o;
}


pub fn single_register_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    eprintln!("{name:15} r{}", register_index_type::read(chunk, &mut offset));
    return offset;
}

pub fn unary_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    eprintln!(
        "{name:15} r{} r{}",
        register_index_type::read(Rc::clone(&chunk), &mut offset),
        register_index_type::read(chunk, &mut offset)
    );
    return offset;
}

pub fn binary_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;

    eprintln!(
        "{name:15} r{} r{} r{}",
        register_index_type::read(Rc::clone(&chunk), &mut offset),
        register_index_type::read(Rc::clone(&chunk), &mut offset),
        register_index_type::read(Rc::clone(&chunk), &mut offset),
    );

    return offset;
}



pub fn stack_access_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let register = register_index_type::read(Rc::clone(&chunk), &mut offset);
    let index = stack_index_type::read(Rc::clone(&chunk), &mut offset);

    eprintln!("{name:15} r{} s[{}]", register, index,);

    return offset;
}

pub fn jmp_if_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let register = register_index_type::read(Rc::clone(&chunk), &mut offset);
    let jmp_addr = instruction_length_type::read(chunk, &mut offset);
    eprintln!("{name:15} r{} addr[{}]", register, jmp_addr);

    return offset;
}

pub fn jmp_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let jmp_addr = instruction_length_type::read(chunk, &mut offset);
    eprintln!("{name:15} addr[{}]", jmp_addr);

    return offset;
}


pub fn fn_call_instruction<T>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let register = register_index_type::read(Rc::clone(&chunk), &mut offset);
    let num_args = chunk.code[offset];
    offset += 1;

    eprintln!("{name:15} r{register} args: {num_args}");

    return offset;
}



pub fn closure_instruction<T: Display>(name: &str, chunk: Rc<Chunk<T>>, mut offset: usize) -> usize {
    offset += 1;
    let fn_register = register_index_type::read(Rc::clone(&chunk), &mut offset);

    let (constant, bytes_read) = Varint::read_bytes(&chunk, offset);
    offset += bytes_read;
    let upvalues_count = chunk.code[offset];
    offset += 1;
    let mut final_string = format!("{name:15} r{fn_register} c{constant} {} (upvalues count: {upvalues_count})", chunk.constants[constant as usize]);
    for _ in 0..(upvalues_count as usize)  {
        let is_local = chunk.code[offset] != 0;
        offset += 1;
        let index = register_index_type::read(Rc::clone(&chunk), &mut offset);


        final_string += &format!("\n\tupvalue (local: {is_local}, index: {index})");
    }

    eprintln!("{}", final_string);
    let f = chunk.constants[constant as usize].as_function().unwrap();

    eprintln!("");
    f.chunk.disassemble(&format!("== {} ==", f.name));
    eprintln!("== ~{} ==", f.name);
    eprintln!("");

    return offset;
}


pub fn disassemble_instruction<T: Display>(chunk: Rc<Chunk<T>>, offset: usize, previous_offset: usize) -> usize {


    eprint!("{offset:04}");
    if offset > 0 && chunk.get_line(offset) == chunk.get_line(previous_offset) {
        eprint!("   | ");
    } else {
        eprint!("{:4} ", chunk.get_line(offset));
    }
    let instruction = chunk.code[offset];
    let instruction = Instructions::from_repr(instruction);

    match instruction {
        Some(Instructions::Return) => simple_instruction("OP_RETURN", offset),
        Some(Instructions::Constant) => constant_instruction("OP_CONSTANT", &chunk, offset),
        Some(Instructions::Load) => constant_register_instruction("OP_LOAD", chunk, offset),
        Some(Instructions::Negate) => unary_instruction("OP_NEGATE", chunk, offset),
        Some(Instructions::Bang) => unary_instruction("OP_BANG", chunk, offset),
        Some(Instructions::Add) => binary_instruction("OP_ADD", chunk, offset),
        Some(Instructions::Sub) => binary_instruction("OP_SUB", chunk, offset),
        Some(Instructions::Mul) => binary_instruction("OP_MUL", chunk, offset),
        Some(Instructions::Div) => binary_instruction("OP_DIV", chunk, offset),
        Some(Instructions::Neq) => binary_instruction("OP_NEQ", chunk, offset),
        Some(Instructions::Eq) => binary_instruction("OP_EQ", chunk, offset),
        Some(Instructions::Lt) => binary_instruction("OP_LT", chunk, offset),
        Some(Instructions::LtEq) => binary_instruction("OP_LTEQ", chunk, offset),
        Some(Instructions::Gt) => binary_instruction("OP_GT", chunk, offset),
        Some(Instructions::GtEq) => binary_instruction("OP_GTEQ", chunk, offset),
        Some(Instructions::Print) => single_register_instruction("OP_PRINT", chunk, offset),
        Some(Instructions::DefineGlobal) => {
            constant_register_instruction("OP_G_DEF", chunk, offset)
        }
        Some(Instructions::GetGlobal) => constant_register_instruction("OP_G_GET", chunk, offset),
        Some(Instructions::SetGlobal) => constant_register_instruction("OP_G_SET", chunk, offset),
        Some(Instructions::PushStack) => simple_instruction("OP_S_PUSH", offset),
        Some(Instructions::PopStack) => simple_instruction("OP_S_POP", offset),
        Some(Instructions::DefineLocal) => single_register_instruction("OP_S_DEF", chunk, offset),
        Some(Instructions::GetLocal) => stack_access_instruction("OP_S_GET", chunk, offset),
        Some(Instructions::SetLocal) => stack_access_instruction("OP_S_SET", chunk, offset),
        Some(Instructions::JumpIfFalse) => jmp_if_instruction("OP_JMP_F", chunk, offset),
        Some(Instructions::Jump) => jmp_instruction("OP_JMP", chunk, offset),
        Some(Instructions::FunctionCall) => fn_call_instruction("OP_FN_CALL", chunk, offset),
        Some(Instructions::FunctionReturn) => simple_instruction("OP_FN_RT", offset),
        Some(Instructions::DebugBreak) => simple_instruction("OP_DEBUG_BREAK", offset),
        Some(Instructions::Closure) => closure_instruction("OP_CLOSURE", chunk, offset),
        Some(Instructions::GetUpvalue) => stack_access_instruction("OP_U_GET", chunk, offset),
        Some(Instructions::SetUpvalue) => stack_access_instruction("OP_U_SET", chunk, offset),
        Some(Instructions::GetField) => constant_register_instruction("OP_F_GET", chunk, offset),
        Some(Instructions::SetField) => constant_set_register_instruction("OP_F_SET", chunk, offset),

        None => offset + 1,
    }
}
