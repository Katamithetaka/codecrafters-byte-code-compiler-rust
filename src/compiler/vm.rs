use std::collections::HashMap;

use crate::{
    compiler::{
        chunk::Chunk,
        instructions::{Instructions, disassemble_instruction},
        value::print_value,
        varint::Varint,
    },
    expressions::{EvaluateError, EvaluateErrorDetails, Value},
};

const STACK_MAX_SIZE: u8 = 255;

pub struct Vm {
    pub ip: usize,
    pub registers: [Value<String>; 256],
    pub global_variables: HashMap<String, Value<String>>,
    pub stack_states: Vec<u8>,
    pub stack: [Value<String>; STACK_MAX_SIZE as usize],
    pub stack_index: u16,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: 0,
            registers: std::array::from_fn(|_| Value::Null),
            global_variables: Default::default(),
            stack_states: vec![],
            stack: std::array::from_fn(|_| Value::Null),
            stack_index: 0,
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

            let v_0 = $vm.registers[register_0 as usize].as_binary_number_op()?;
            let v_1 = $vm.registers[register_1 as usize].as_binary_number_op()?;

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

            let v_0 = $vm.registers[register_0 as usize].as_binary_number_op()?;
            let v_1 = $vm.registers[register_1 as usize].as_binary_number_op()?;

            $vm.registers[dst_register as usize] = Value::Boolean(v_0 $op v_1);
            print_value(&$vm.registers[dst_register as usize]);
        }
    };
}

pub fn execute_instruction(
    vm: &mut Vm,
    chunk: &Chunk,
    instruction: Instructions,
) -> Result<(), InterpretError> {
    match instruction {
        Instructions::Constant => {
            let (value, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            if DEBUG_TRACE_EXECUTION {
                print_value(&chunk.constants[value as usize]);
            }
        }
        Instructions::Load => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            vm.registers[register as usize] = chunk.constants[constant as usize].clone().into();
        }
        Instructions::Negate => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            let dst_register = chunk.code[vm.ip];
            vm.ip += 1;
            let v = vm.registers[register as usize].as_number()?;
            vm.registers[dst_register as usize] = Value::Number(-v);
            print_value(&vm.registers[dst_register as usize]);
        }
        Instructions::Bang => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            let dst_register = chunk.code[vm.ip];
            vm.ip += 1;

            vm.registers[dst_register as usize] =
                Value::Boolean(!vm.registers[register as usize].is_truthy());
            print_value(&vm.registers[dst_register as usize]);
        }
        Instructions::Add => {
            let register_0 = chunk.code[vm.ip];
            vm.ip += 1;
            let register_1 = chunk.code[vm.ip];
            vm.ip += 1;
            let dst_register = chunk.code[vm.ip];
            vm.ip += 1;
            let v_0 = &vm.registers[register_0 as usize];
            let v_1 = &vm.registers[register_1 as usize];
            vm.registers[dst_register as usize] = match (v_0, v_1) {
                (crate::expressions::Value::Number(a), crate::expressions::Value::Number(b)) => {
                    Value::Number(a + b)
                }
                (crate::expressions::Value::String(a), crate::expressions::Value::String(b)) => {
                    Value::String(format!("{a}{b}"))
                }
                (_, _) => return Err(InterpretError::UnmatchedTypes),
            };
            print_value(&vm.registers[dst_register as usize]);
        }
        Instructions::Sub => binary_op!(chunk, vm, -),
        Instructions::Mul => binary_op!(chunk, vm, *),
        Instructions::Div => binary_op!(chunk, vm, /),
        Instructions::Eq => eq_op!(chunk, vm, ==),
        Instructions::Neq => eq_op!(chunk, vm, !=),
        Instructions::Lt => cmp_op!(chunk, vm, <),
        Instructions::Gt => cmp_op!(chunk, vm, >),
        Instructions::LtEq => cmp_op!(chunk, vm, <=),
        Instructions::GtEq => cmp_op!(chunk, vm, >=),
        Instructions::Return => return Ok(()),
        Instructions::Print => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            println!("{}", vm.registers[register as usize]);
        }
        Instructions::DefineGlobal => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            let v = &chunk.constants[constant as usize];
            let register_v = std::mem::replace(&mut vm.registers[register as usize], Value::Null);
            match v {
                crate::expressions::Value::String(a) => {
                    vm.global_variables.insert(a.to_string(), register_v)
                }
                _ => return Err(InterpretError::InvalidIdentifierType),
            };
        }
        Instructions::GetGlobal => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            let v = &chunk.constants[constant as usize];
            vm.registers[register as usize] = match v {
                Value::String(a) => vm
                    .global_variables
                    .get(*a)
                    .ok_or_else(|| InterpretError::UndefinedVariable(a.to_string()))?
                    .clone(),
                _ => {
                    return Err(InterpretError::InvalidIdentifierType);
                }
            };
        }
        Instructions::SetGlobal => {
            let register = chunk.code[vm.ip];
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            let v = &chunk.constants[constant as usize];
            let ident = v.as_string()?;

            let variable = vm
                .global_variables
                .get_mut(*ident)
                .ok_or_else(|| InterpretError::UndefinedVariable(ident.to_string()))?;

            *variable = vm.registers[register as usize].clone();
        }
        Instructions::PushStack => {
            vm.stack_states.push(vm.stack_index as u8);
        }
        Instructions::PopStack => {
            vm.stack_index = match vm.stack_states.pop() {
                Some(previous_index) => previous_index as u16,
                None => return Err(InterpretError::InvalidStackPop),
            };
        }
        Instructions::DefineLocal => {
            let last = match vm.stack_states.last() {
                Some(v) => v,
                None => return Err(InterpretError::LocalInGlobal),
            };

            if DEBUG_TRACE_EXECUTION {
                let depth = vm.stack_states.len();
                let index = vm.stack_index as u8 - last;
                eprintln!("Writing local {depth} {index}");
            }

            let register = chunk.code[vm.ip];
            vm.ip += 1;

            let register_v = std::mem::replace(&mut vm.registers[register as usize], Value::Null);
            vm.stack[vm.stack_index as usize] = register_v;

            vm.stack_index = match vm.stack_index.checked_add(1) {
                Some(v) => v,
                None => return Err(InterpretError::StackOverflow),
            };

            if (vm.stack_index as u8) >= STACK_MAX_SIZE {
                return Err(InterpretError::StackOverflow);
            }
        }
        Instructions::GetLocal => {
            let output_register = chunk.code[vm.ip];
            vm.ip += 1;

            let depth = chunk.code[vm.ip] as usize;
            vm.ip += 1;

            let index = chunk.code[vm.ip] as usize;
            vm.ip += 1;

            if DEBUG_TRACE_EXECUTION {
                eprintln!("Getting local {depth} {index}");
            }

            if depth > vm.stack_states.len() {
                return Err(EvaluateErrorDetails::UndefinedVariable(format!(
                    "Local {depth} {index}: Depth is too high"
                )));
            }

            let depth = vm.stack_states[depth - 1] as usize;
            let index = depth + index;

            if index >= vm.stack_index as usize {
                return Err(EvaluateErrorDetails::UndefinedVariable(format!(
                    "Stack({depth}, {}) {index}: Index is too high",
                    index - depth
                )));
            }

            vm.registers[output_register as usize] = vm.stack[index as usize].clone();
        }
        Instructions::SetLocal => {
            let output_register = chunk.code[vm.ip];
            vm.ip += 1;

            let depth = chunk.code[vm.ip] as usize;
            vm.ip += 1;

            let index = chunk.code[vm.ip] as usize;
            vm.ip += 1;

            if DEBUG_TRACE_EXECUTION {
                eprintln!("Getting local {depth} {index}");
            }

            if depth > vm.stack_states.len() {
                return Err(EvaluateErrorDetails::UndefinedVariable(format!(
                    "Local {depth} {index}: Depth is too high"
                )));
            }

            let depth = vm.stack_states[depth - 1] as usize;
            let index = depth + index;

            if index >= vm.stack_index as usize {
                return Err(EvaluateErrorDetails::UndefinedVariable(format!(
                    "Stack({depth}, {}) {index}: Index is too high",
                    index - depth
                )));
            }

            vm.stack[index as usize] = vm.registers[output_register as usize].clone();
        }
    }

    Ok(())
}

const DEBUG_TRACE_EXECUTION: bool = false;
pub fn interpret(chunk: &Chunk) -> Result<(), EvaluateError> {
    let mut vm = Vm::new();
    let mut previous_ip = 0;
    while vm.ip < chunk.code.len() {
        let instruction = chunk.code[vm.ip];
        if DEBUG_TRACE_EXECUTION {
            let tmp = vm.ip;
            disassemble_instruction(&chunk, vm.ip, previous_ip);
            previous_ip = tmp;
        }

        vm.ip += 1;

        let result = match Instructions::from_repr(instruction) {
            Some(v) => execute_instruction(&mut vm, chunk, v),
            None => Err(InterpretError::UnexpectedOpCode(instruction)),
        };

        match result {
            Ok(_) => {
                continue;
            }
            Err(err) => {
                return Err(EvaluateError {
                    error: err,
                    line: chunk.get_line(vm.ip as usize) as usize,
                });
            }
        }
    }

    return Ok(());
}
