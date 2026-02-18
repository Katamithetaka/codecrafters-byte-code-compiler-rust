use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::{
        chunk::Chunk,
        instructions::{Instructions, disassemble_instruction},
        value::print_value,
        varint::Varint,
    },
    expressions::{EvaluateError, EvaluateErrorDetails}, value::{Closure, class, class_instance::ClassInstance},
};


pub use crate::expressions::Value;

const STACK_MAX_SIZE: u32 = u16::MAX as u32;
const REGISTER_MAX_SIZE: usize = 256;
const DEBUG_TRACE_EXECUTION: bool = false;
type Registers = [Value<String>; REGISTER_MAX_SIZE];

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_ip: usize,
    pub register_base: u8,
    pub stack_index: u16,
    pub stack_state_index: usize,
    pub chunk: Rc<Chunk<String>>,
    pub closure: Closure<String>
}



pub struct Vm {
    pub ip: usize,
    pub registers: Registers,
    pub global_variables: HashMap<String, Value<String>>,
    pub stack_states: Vec<u16>,
    pub stack: Vec<Value<String>>,
    pub stack_index: u32,
    pub call_stack: Vec<CallFrame>,
    pub current_chunk: Rc<Chunk<String>>,

}

impl Vm {
    pub fn get_register(&self, register: u8) -> u8 {
        let frame_base = self.call_stack.last()
            .map(|f| f.register_base)
            .unwrap_or(0);
        // Return the absolute stack position: frame base + local offset
        frame_base + register
    }

    pub fn get_stack_index(&self, stack_index: u16) -> u16 {
        // Get the current call frame's stack base (or 0 if at top level)
        let frame_base = self.call_stack.last()
            .map(|f| f.stack_index)
            .unwrap_or(0);
        // Return the absolute stack position: frame base + local offset
        frame_base + stack_index
    }
}

pub fn save_registers(registers: &mut Registers) -> Registers {
    std::mem::replace(registers, std::array::from_fn(|_| Value::Null))
}

impl Vm {
    pub fn new(chunk: Rc<Chunk<String>>) -> Self {
        Self {
            ip: 0,
            registers: std::array::from_fn(|_| Value::Null),
            global_variables: Default::default(),
            stack_states: vec![],
            stack: vec![Value::Null; STACK_MAX_SIZE as usize],
            stack_index: 0,
            call_stack: vec![],
            current_chunk: chunk
        }
    }
}

pub type InterpretError = crate::expressions::EvaluateErrorDetails;

macro_rules! binary_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;
            let register_1 = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;
            let dst_register = $vm.get_register($chunk.code[$vm.ip]);

            $vm.ip += 1;

            let v_0 = $vm.registers[register_0 as usize].as_binary_number_op()?;
            let v_1 = $vm.registers[register_1 as usize].as_binary_number_op()?;

            $vm.registers[dst_register as usize] = Value::Number(v_0 $op v_1);
            if DEBUG_TRACE_EXECUTION {
                print_value(&$vm.registers[dst_register as usize]);
            }

        }
    };
}

macro_rules! eq_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;
            let register_1 = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;
            let dst_register = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;



            $vm.registers[dst_register as usize] = Value::Boolean($vm.registers[register_0 as usize] $op $vm.registers[register_1 as usize]);
            if DEBUG_TRACE_EXECUTION {
                print_value(&$vm.registers[dst_register as usize]);
            }

        }
    };
}

macro_rules! cmp_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;
            let register_1 = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;
            let dst_register = $vm.get_register($chunk.code[$vm.ip]);
            $vm.ip += 1;

            let v_0 = $vm.registers[register_0 as usize].as_binary_number_op()?;
            let v_1 = $vm.registers[register_1 as usize].as_binary_number_op()?;

            $vm.registers[dst_register as usize] = Value::Boolean(v_0 $op v_1);
            if DEBUG_TRACE_EXECUTION {
                print_value(&$vm.registers[dst_register as usize]);
            }

        }
    };
}

pub fn execute_instruction(
    vm: &mut Vm,
    instruction: Instructions,
) -> Result<(), InterpretError> {
    let chunk = &vm.current_chunk;
    match instruction {
        Instructions::Constant => {
            let (value, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            if DEBUG_TRACE_EXECUTION {
                print_value(&chunk.constants[value as usize]);
            }
        }
        Instructions::Load => {
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            vm.registers[register as usize] = chunk.constants[constant as usize].clone().into();
        }
        Instructions::Negate => {
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let dst_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let v = vm.registers[register as usize].as_number()?;
            vm.registers[dst_register as usize] = Value::Number(-v);
            if DEBUG_TRACE_EXECUTION {
                print_value(&vm.registers[dst_register as usize]);
            }
        }
        Instructions::Bang => {
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let dst_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;

            vm.registers[dst_register as usize] =
                Value::Boolean(!vm.registers[register as usize].is_truthy());
            if DEBUG_TRACE_EXECUTION {
                print_value(&vm.registers[dst_register as usize]);
            }
        }
        Instructions::Add => {
            let register_0 = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let register_1 = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let dst_register = vm.get_register(chunk.code[vm.ip]);
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
            if DEBUG_TRACE_EXECUTION {
                print_value(&vm.registers[dst_register as usize]);
            }
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
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            println!("{}", vm.registers[register as usize]);
        }
        Instructions::DefineGlobal => {
            let register = vm.get_register(chunk.code[vm.ip]);
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
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            let v = &chunk.constants[constant as usize];
            vm.registers[register as usize] = match v {
                Value::String(a) => vm
                    .global_variables
                    .get(a)
                    .ok_or_else(|| InterpretError::UndefinedVariable(a.to_string()))?
                    .clone(),
                _ => {
                    return Err(InterpretError::InvalidIdentifierType);
                }
            };
        }
        Instructions::SetGlobal => {
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;
            let v = &chunk.constants[constant as usize];
            let ident = v.as_string()?;

            let variable = vm
                .global_variables
                .get_mut(&ident)
                .ok_or_else(|| InterpretError::UndefinedVariable(ident.to_string()))?;

            *variable = vm.registers[register as usize].clone();
        }
        Instructions::PushStack => {
            vm.stack_states.push(vm.stack_index as u16);
        }
        Instructions::PopStack => {
            vm.stack_index = match vm.stack_states.pop() {
                Some(previous_index) => previous_index as u32,
                None => return Err(InterpretError::InvalidStackPop),
            };
        }
        Instructions::DefineLocal => {
            if DEBUG_TRACE_EXECUTION {
                eprintln!("Writing local {}", vm.stack_index);
            }

            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;

            let register_v = std::mem::replace(&mut vm.registers[register as usize], Value::Null);
            vm.stack[vm.stack_index as usize] = register_v;

            vm.stack_index = match vm.stack_index.checked_add(1) {
                Some(v) => v,
                None => return Err(InterpretError::StackOverflow),
            };

            if (vm.stack_index) >= STACK_MAX_SIZE {
                vm.stack_index;
                return Err(InterpretError::StackOverflow);
            }
        }
        Instructions::GetLocal => {
            let output_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;

            let index = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip+1]]);
            vm.ip += 2;




            let index = vm.get_stack_index(index) as usize;

            if DEBUG_TRACE_EXECUTION {
                eprintln!("Getting local {}",  index);
            }

            if index >= vm.stack_index as usize {
                return Err(EvaluateErrorDetails::UndefinedVariable(format!(
                    "Stack {index}: Index is too high",
                )));
            }

            vm.registers[output_register as usize] = vm.stack[index as usize].clone();
        }
        Instructions::SetLocal => {
            let output_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;


            let index = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip+1]]);
            vm.ip += 2;

            let index = vm.get_stack_index(index) as usize;

            if DEBUG_TRACE_EXECUTION {
                eprintln!("Getting local {index}");
            }

            if index >= vm.stack_index as usize {
                return Err(EvaluateErrorDetails::UndefinedVariable(format!(
                    "Stack {index}: Index is too high",
                )));
            }

            vm.stack[index as usize].set(vm.registers[output_register as usize].clone());
        }
        Instructions::JumpIfFalse => {
            let register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let jmp_addr = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip + 1]]);
            vm.ip += 2;

            if !vm.registers[register as usize].is_truthy() {
                vm.ip = jmp_addr as usize;
            }
        }
        Instructions::Jump => {
            let jmp_addr = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip + 1]]);
            vm.ip += 2;

            vm.ip = jmp_addr as usize;
        }
        Instructions::FunctionCall => {
            let fn_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;

            let num_args = chunk.code[vm.ip];
            vm.ip += 1;

            let func_val = &vm.registers[fn_register as usize].inner();

            let mut call_closure = |c: &Closure<String>| {
                if c.function.arguments_count() != num_args as u8 {
                    return Err(EvaluateErrorDetails::InvalidArgCount);
                }

                // Push a call frame
                vm.call_stack.push(CallFrame {
                    chunk: vm.current_chunk.clone(),
                    closure: c.clone(),
                    return_ip: vm.ip,
                    register_base: fn_register,
                    stack_index: vm.stack_index as u16 - num_args as u16,
                    stack_state_index: vm.stack_states.len()
                });

                vm.stack_states.push(vm.stack_index as u16);

                // switch to closure's chunk
                vm.current_chunk = c.chunk.clone();
                vm.ip = 0;

                return Ok(())
            };

            match func_val {

                Value::Closure(c) => {
                    call_closure(c)?;
                }


                Value::GlobalFunction(gf) => {
                    if gf.arguments_count != num_args {
                        return Err(EvaluateErrorDetails::InvalidArgCount);
                    }

                    // collect arguments from the stack
                    let args_start = vm.stack_index as usize - num_args as usize;
                    let args: Vec<_> = if args_start < vm.stack_index as usize {
                        let slice = &mut vm.stack[args_start..vm.stack_index as usize];
                        slice.iter_mut().map(std::mem::take).collect()
                    } else { vec![] };

                    // call the Rust-native function
                    let return_val = (gf.callable)(args);

                    // store result in the function register
                    vm.registers[fn_register as usize] = return_val;

                    // reset stack index after popping args
                    vm.stack_index -= num_args as u32;
                },
                Value::Class(c) => {
                    let class_instance = ClassInstance::new(c.clone());

                    // todo: call constructor
                    vm.registers[fn_register as usize] = Value::Instance(class_instance);
                }

                _ => return Err(EvaluateErrorDetails::ExpectedFunction),
            }
        },
        Instructions::FunctionReturn => {
            let return_val = std::mem::take(&mut vm.registers[vm.get_register(0) as usize]);
            let v = vm.call_stack.pop().ok_or(EvaluateErrorDetails::InvalidReturnStatement)?;
            vm.ip = v.return_ip;
            vm.current_chunk = v.chunk;
            vm.registers[v.register_base as usize] = return_val;
            vm.stack_index = v.stack_index as u32;

            vm.stack_states.truncate(v.stack_state_index as usize);

        },
        Instructions::DebugBreak => {
            for (i, v) in vm.registers.iter().enumerate().filter(|(_, i)| !i.is_null()) {
                println!("r{i} = {v}");
            }

            std::io::stdin().read_line(&mut String::new()).map_err(|_| EvaluateErrorDetails::StdinFailed)?;
        },
        Instructions::Closure => {
            let dst_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;
            let (constant, size) = Varint::read_bytes(chunk, vm.ip);
            vm.ip += size;

            let func = &chunk.constants[constant as usize];

            let upvalue_count = chunk.code[vm.ip];
            vm.ip += 1;

            let mut upvalues = vec![];
            for _ in 0..upvalue_count {
                let is_local = chunk.code[vm.ip] != 0;
                vm.ip += 1;
                let index = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip + 1]]);
                vm.ip += 2;

                if is_local {
                    let index = vm.get_stack_index(index) as usize;

                    vm.stack[index as usize] = vm.stack[index as usize].to_cell();
                    upvalues.push(vm.stack[index as usize].clone());
                } else {
                    upvalues.push(vm.call_stack.last().unwrap().closure.upvalues[index as usize].clone());
                }
            }

            let func = func.as_function()?;
            vm.registers[dst_register as usize] = Value::Closure(Closure {
                chunk: func.chunk(),
                function: func,
                upvalues,
            });
        },
        Instructions::GetUpvalue => {
            let output_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;

            let upvalue_index = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip + 1]]);
            vm.ip += 2;

            // Get the upvalue from the current closure
            let current_closure = &vm.call_stack.last()
                .ok_or(EvaluateErrorDetails::InvalidUpvalueAccess)?
                .closure;

            let upvalue = &current_closure.upvalues[upvalue_index as usize];

            // Unwrap the Cell to get the actual value
            match upvalue {
                Value::Cell(cell) => {
                    vm.registers[output_register as usize] = cell.borrow().clone();
                }
                _ => {
                    return Err(EvaluateErrorDetails::InvalidUpvalueType);
                }
            }
        }

        Instructions::SetUpvalue => {
            let input_register = vm.get_register(chunk.code[vm.ip]);
            vm.ip += 1;

            let upvalue_index = u16::from_be_bytes([chunk.code[vm.ip], chunk.code[vm.ip + 1]]);
            vm.ip += 2;

            // Get the upvalue from the current closure
            let current_closure = &vm.call_stack.last()
                .ok_or(EvaluateErrorDetails::InvalidUpvalueAccess)?
                .closure;

            let upvalue = &current_closure.upvalues[upvalue_index as usize];

            // Update the Cell with the new value
            match upvalue {
                Value::Cell(cell) => {
                    *cell.borrow_mut() = vm.registers[input_register as usize].clone();
                }
                _ => {
                    return Err(EvaluateErrorDetails::InvalidUpvalueType);
                }
            }
        }

    }

    Ok(())
}

pub fn interpret_with_vm(vm: &mut Vm)  -> Result<(), EvaluateError> {
    let mut previous_ip = 0;
    while vm.ip < vm.current_chunk.code.len() {
        let instruction = vm.current_chunk.code[vm.ip];
        if DEBUG_TRACE_EXECUTION {
            let tmp = vm.ip;
            disassemble_instruction(&vm.current_chunk, vm.ip, previous_ip);
            previous_ip = tmp;
        }

        vm.ip += 1;

        let result = match Instructions::from_repr(instruction) {
            Some(v) => execute_instruction(vm, v),
            None => Err(InterpretError::UnexpectedOpCode(instruction)),
        };

        match result {
            Ok(_) => {
                continue;
            }
            Err(err) => {
                return Err(EvaluateError {
                    error: err,
                    line: vm.current_chunk.get_line(vm.ip as usize) as usize,
                });
            }
        }
    }

    return Ok(());
}

pub fn interpret(chunk: Rc<Chunk<String>>) -> Result<(), EvaluateError> {
    let mut vm = Box::new(Vm::new(chunk));
    interpret_with_vm(&mut vm)
}
