use crate::compiler::{chunk::Chunk, value::print_value, varint::Varint};

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
}

pub fn simple_instruction(name: &str, offset: usize) -> usize {
    eprintln!("{name:15}");
    return offset + 1;
}

pub fn print_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    eprintln!("{name:15} r{}", chunk.code[offset + 1]);
    return offset + 2;
}

pub fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let (constant, o) = Varint::read_bytes(chunk, offset + 1);
    eprint!("{name:15} c{constant} ");
    print_value(&chunk.constants[constant as usize]);

    return offset + o + 1;
}

pub fn constant_register_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let (constant, o) = Varint::read_bytes(chunk, offset + 2);

    eprint!("{name:15} r{} c{} ", chunk.code[offset + 1], constant);
    print_value(&chunk.constants[constant as usize]);

    return offset + 2 + o;
}

pub fn single_register_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    eprintln!("{name:15} r{}", chunk.code[offset + 1],);
    return offset + 2;
}

pub fn unary_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    eprintln!(
        "{name:15} r{} r{}",
        chunk.code[offset + 1],
        chunk.code[offset + 2]
    );
    return offset + 3;
}

pub fn binary_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    eprintln!(
        "{name:15} r{} r{} r{}",
        chunk.code[offset + 1],
        chunk.code[offset + 2],
        chunk.code[offset + 3],
    );
    return offset + 4;
}

pub fn stack_access(chunk: &Chunk, offset: usize) -> (u16, usize) {
    return (u16::from_be_bytes([chunk.code[offset], chunk.code[offset+1]]), 2);
}

pub fn stack_access_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let register = chunk.code[offset + 1];
    let (index, o) = stack_access(chunk, offset + 2);

    eprintln!("{name:15} r{} s[{}]", register, index,);

    return offset + o + 2;
}

pub fn jmp_if_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let register = chunk.code[offset + 1];
    let jmp_addr = u16::from_be_bytes([chunk.code[offset + 2], chunk.code[offset + 3]]);
    eprintln!("{name:15} r{} addr[{}]", register, jmp_addr);

    return offset + 4;
}

pub fn jmp_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let jmp_addr = u16::from_be_bytes([chunk.code[offset + 1], chunk.code[offset + 2]]);
    eprintln!("{name:15} addr[{}]", jmp_addr);

    return offset + 3;
}


pub fn fn_call_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let fn_register = chunk.code[offset + 1];
    let num_args = chunk.code[offset + 2];
    eprintln!("{name:15} r{fn_register} args: {num_args}");

    return offset + 3;
}


pub fn disassemble_instruction(chunk: &Chunk, offset: usize, previous_offset: usize) -> usize {
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
        Some(Instructions::Constant) => constant_instruction("OP_CONSTANT", chunk, offset),
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
        Some(Instructions::Print) => print_instruction("OP_PRINT", chunk, offset),
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

        None => offset + 1,
    }
}
