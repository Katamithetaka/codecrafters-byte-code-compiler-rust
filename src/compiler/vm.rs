use std::io::Error;

use crate::compiler::{
    chunk::Chunk,
    instructions::{self, Instructions, disassemble_instruction},
    value::{Value, print_value},
    varint::Varint,
};

pub struct Vm {
    pub ip: usize,
    pub registers: [Value; 256],
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: 0,
            registers: std::array::from_fn(|_| Value::Null),
        }
    }
}

pub type InterpretError = crate::expressions::EvaluateErrorDetails;

macro_rules! binary_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $chunk.code[$vm.ip];
            $vm.ip += 1;
            let register_1 = $chunk.code[$vm.ip];
            $vm.ip += 1;
            let dst_register = $chunk.code[$vm.ip];
            $vm.ip += 1;

            let v_0 = match $vm.registers[register_0 as usize] {
                Value::Number(v) => v,
                _ => return Err(InterpretError::BinaryNumberOp),
            };
            let v_1 = match $vm.registers[register_1 as usize] {
                Value::Number(v) => v,
                _ => return Err(InterpretError::BinaryNumberOp),
            };

            $vm.registers[dst_register as usize] = Value::Number(v_0 $op v_1);
            print_value(&$vm.registers[dst_register as usize]);
        }
    };
}

macro_rules! eq_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $chunk.code[$vm.ip];
            $vm.ip += 1;
            let register_1 = $chunk.code[$vm.ip];
            $vm.ip += 1;
            let dst_register = $chunk.code[$vm.ip];
            $vm.ip += 1;



            $vm.registers[dst_register as usize] = Value::Boolean($vm.registers[register_0 as usize] $op $vm.registers[register_1 as usize]);
            print_value(&$vm.registers[dst_register as usize]);
        }
    };
}

macro_rules! cmp_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $chunk.code[$vm.ip];
            $vm.ip += 1;
            let register_1 = $chunk.code[$vm.ip];
            $vm.ip += 1;
            let dst_register = $chunk.code[$vm.ip];
            $vm.ip += 1;

            let v_0 = match $vm.registers[register_0 as usize] {
                Value::Number(v) => v,
                _ => return Err(InterpretError::BinaryNumberOp),
            };
            let v_1 = match $vm.registers[register_1 as usize] {
                Value::Number(v) => v,
                _ => return Err(InterpretError::BinaryNumberOp),
            };

            $vm.registers[dst_register as usize] = Value::Boolean(v_0 $op v_1);
            print_value(&$vm.registers[dst_register as usize]);
        }
    };
}

const DEBUG_TRACE_EXECUTION: bool = false;
pub fn interpret(chunk: &Chunk) -> Result<(), InterpretError> {
    let mut vm = Vm::new();
    while vm.ip < chunk.code.len() {
        let instruction = chunk.code[vm.ip];
        if DEBUG_TRACE_EXECUTION {
            disassemble_instruction(&chunk, vm.ip);
        }
        vm.ip += 1;
        match Instructions::from_repr(instruction) {
            Some(Instructions::Constant) => {
                let (constant, size) = Varint::read_bytes(chunk, vm.ip);
                vm.ip += size;
                let value = chunk.value_array[constant as usize].clone();
                print_value(&value);
            }
            Some(Instructions::Load) => {
                let register = chunk.code[vm.ip];
                vm.ip += 1;
                let (constant, size) = Varint::read_bytes(chunk, vm.ip);
                vm.ip += size;
                vm.registers[register as usize] = chunk.value_array[constant as usize].clone();
            }
            Some(Instructions::Negate) => {
                let register = chunk.code[vm.ip];
                vm.ip += 1;
                let dst_register = chunk.code[vm.ip];
                vm.ip += 1;
                let v = match vm.registers[register as usize] {
                    Value::Number(v) => v,
                    _ => return Err(InterpretError::UnaryNumberOp),
                };
                vm.registers[dst_register as usize] = Value::Number(-v);
                print_value(&vm.registers[dst_register as usize]);
            }
            Some(Instructions::Bang) => {
                let register = chunk.code[vm.ip];
                vm.ip += 1;
                let dst_register = chunk.code[vm.ip];
                vm.ip += 1;

                vm.registers[dst_register as usize] =
                    Value::Boolean(!vm.registers[register as usize].is_truthy());
                print_value(&vm.registers[dst_register as usize]);
            }
            Some(Instructions::Add) => {
                let register_0 = chunk.code[vm.ip];
                vm.ip += 1;
                let register_1 = chunk.code[vm.ip];
                vm.ip += 1;
                let dst_register = chunk.code[vm.ip];
                vm.ip += 1;
                let v_0 = &vm.registers[register_0 as usize];
                let v_1 = &vm.registers[register_1 as usize];
                vm.registers[dst_register as usize] = match (v_0, v_1) {
                    (
                        crate::expressions::Value::Number(a),
                        crate::expressions::Value::Number(b),
                    ) => Value::Number((a + b)),
                    (
                        crate::expressions::Value::String(a),
                        crate::expressions::Value::String(b),
                    ) => Value::String(format!("{a}{b}")),
                    (_, _) => return Err(InterpretError::BinaryNumberOp),
                };
                print_value(&vm.registers[dst_register as usize]);
            }
            Some(Instructions::Sub) => binary_op!(chunk, vm, -),
            Some(Instructions::Mul) => binary_op!(chunk, vm, *),
            Some(Instructions::Div) => binary_op!(chunk, vm, /),
            Some(Instructions::Eq) => eq_op!(chunk, vm, ==),
            Some(Instructions::Neq) => eq_op!(chunk, vm, !=),
            Some(Instructions::Lt) => cmp_op!(chunk, vm, <),
            Some(Instructions::Gt) => cmp_op!(chunk, vm, >),
            Some(Instructions::LtEq) => cmp_op!(chunk, vm, <=),
            Some(Instructions::GtEq) => cmp_op!(chunk, vm, >=),
            Some(Instructions::Return) => return Ok(()),
            Some(Instructions::Print) => {
                let register = chunk.code[vm.ip];
                vm.ip += 1;
                println!("{}", vm.registers[register as usize]);
            }
            None => return Err(InterpretError::UnexpectedOpCode(instruction)),
        }
    }

    return Ok(());
}
