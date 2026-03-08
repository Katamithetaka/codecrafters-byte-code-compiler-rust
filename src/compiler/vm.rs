use std::{collections::HashMap, rc::Rc};

use crate::{
    compiler::{
        chunk::Chunk, instructions::{Instructions, disassemble_instruction}, int_types::{instruction_length_type, line_type, register_index_type, stack_index_type, stack_pointer_type, varint_type}, varint::Varint
    },
    expressions::{EvaluateError, EvaluateErrorDetails}, value::{Closure, Function, callable::{Callable, FunctionKind}, class_instance::ClassInstance},
};

pub use crate::compiler::int_types::VmRead;

pub use crate::expressions::Value;

const STACK_MAX_SIZE: stack_pointer_type = stack_index_type::MAX as stack_pointer_type;
const REGISTER_MAX_SIZE: usize = 256;
const DEBUG_TRACE_EXECUTION: bool = false;
type Registers = [Value<String>; REGISTER_MAX_SIZE];

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_ip: usize,
    pub register_base: register_index_type,
    pub stack_index: stack_index_type,
    pub stack_state_index: usize,
    pub chunk: Rc<Chunk<String>>,
    pub closure: Closure<String>
}



pub struct Vm {
    pub ip: usize,
    pub registers: Registers,
    pub global_variables: HashMap<String, Value<String>>,
    pub stack_states: Vec<stack_index_type>,
    pub stack: Vec<Value<String>>,
    pub stack_index: stack_pointer_type,
    pub call_stack: Vec<CallFrame>,
    pub current_chunk: Rc<Chunk<String>>,

}

impl Vm {
    pub fn get_register(&self, register: register_index_type) -> register_index_type {
        let frame_base = self.call_stack.last()
            .map(|f| f.register_base)
            .unwrap_or(0);
        // Return the absolute stack position: frame base + local offset
        frame_base + register
    }

    pub fn get_stack_index(&self, stack_index: stack_index_type) -> stack_index_type {
        // Get the current call frame's stack base (or 0 if at top level)
        let frame_base = self.call_stack.last()
            .map(|f| f.stack_index)
            .unwrap_or(0);
        // Return the absolute stack position: frame base + local offset
        frame_base + stack_index
    }

    pub fn read_register(&mut self) -> register_index_type {
        assert!(self.ip < self.current_chunk.code.len());
        let register = register_index_type::read_bytes(self);
        return self.get_register(register);
    }

    pub fn read_constant(&mut self) -> varint_type {
        let (constant, size) = Varint::read_bytes(&self.current_chunk, self.ip);
        self.ip += size;
        return constant;
    }

    pub fn read_stack_index(&mut self) -> stack_index_type {
        return stack_index_type::read_bytes(self);

    }

    pub fn define_local(&mut self, value: Value<String>) -> Result<(), InterpretError> {
        if DEBUG_TRACE_EXECUTION {
            eprintln!("Writing local {}", self.stack_index);
        }


        self.stack[self.stack_index as usize] = value;

        self.stack_index = match self.stack_index.checked_add(1) {
            Some(v) => v,
            None => return Err(InterpretError::StackOverflow),
        };

        if (self.stack_index) >= STACK_MAX_SIZE {
            self.stack_index;
            return Err(InterpretError::StackOverflow);
        }

        Ok(())
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
            let register_0 = $vm.read_register();
            let register_1 = $vm.read_register();
            let dst_register = $vm.read_register();

            let v_0 = $vm.registers[register_0 as usize].as_binary_number_op()?;
            let v_1 = $vm.registers[register_1 as usize].as_binary_number_op()?;

            $vm.registers[dst_register as usize] = Value::Number(v_0 $op v_1);
            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &$vm.registers[dst_register as usize]);
            }

        }
    };
}

macro_rules! eq_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $vm.read_register();
            let register_1 = $vm.read_register();
            let dst_register = $vm.read_register();



            $vm.registers[dst_register as usize] = Value::Boolean($vm.registers[register_0 as usize] $op $vm.registers[register_1 as usize]);
            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &$vm.registers[dst_register as usize]);
            }

        }
    };
}

macro_rules! cmp_op {
    ($chunk: ident, $vm: ident, $op:tt) => {
        {
            let register_0 = $vm.read_register();
            let register_1 = $vm.read_register();
            let dst_register = $vm.read_register();

            let v_0 = $vm.registers[register_0 as usize].as_binary_number_op()?;
            let v_1 = $vm.registers[register_1 as usize].as_binary_number_op()?;

            $vm.registers[dst_register as usize] = Value::Boolean(v_0 $op v_1);
            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &$vm.registers[dst_register as usize]);
            }

        }
    };
}

pub fn execute_instruction(
    vm: &mut Vm,
    instruction: Instructions,
) -> Result<(), InterpretError> {
    let chunk = &Rc::clone(&vm.current_chunk);
    match instruction {
        Instructions::Constant => {
            let value = vm.read_constant();

            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &chunk.constants[value as usize]);
            }
        }
        Instructions::Load => {
            let register = vm.read_register();
            let constant = vm.read_constant();

            vm.registers[register as usize] = chunk.constants[constant as usize].clone().into();
        }
        Instructions::Negate => {
            let register = vm.read_register();
            let dst_register = vm.read_register();
            let v = vm.registers[register as usize].as_number()?;
            vm.registers[dst_register as usize] = Value::Number(-v);
            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &vm.registers[dst_register as usize]);
            }
        }
        Instructions::Bang => {
            let register = vm.read_register();
            let dst_register = vm.read_register();

            vm.registers[dst_register as usize] =
                Value::Boolean(!vm.registers[register as usize].is_truthy());
            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &vm.registers[dst_register as usize]);
            }
        }
        Instructions::Add => {
            let register_0 = vm.read_register();
            let register_1 = vm.read_register();
            let dst_register = vm.read_register();
            let v_0 = &vm.registers[register_0 as usize];
            let v_1 = &vm.registers[register_1 as usize];
            vm.registers[dst_register as usize] = match (v_0.as_add_op()?, v_1.as_add_op()?) {
                (crate::expressions::Value::Number(a), crate::expressions::Value::Number(b)) => {
                    Value::Number(a + b)
                }
                (crate::expressions::Value::String(a), crate::expressions::Value::String(b)) => {
                    Value::String(format!("{a}{b}"))
                }
                (_, _) => return Err(InterpretError::UnmatchedTypes),
            };
            if DEBUG_TRACE_EXECUTION {
                eprintln!("{}", &vm.registers[dst_register as usize]);
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
            let register = vm.read_register();
            println!("{}", vm.registers[register as usize]);
        }
        Instructions::DefineGlobal => {
            let register = vm.read_register();
            let constant = vm.read_constant();

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
            let register = vm.read_register();
            let constant = vm.read_constant();

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
            let register = vm.read_register();
            let constant = vm.read_constant();

            let v = &chunk.constants[constant as usize];
            let ident = v.as_string()?;

            let variable = vm
                .global_variables
                .get_mut(&ident)
                .ok_or_else(|| InterpretError::UndefinedVariable(ident.to_string()))?;

            *variable = vm.registers[register as usize].clone();
        }
        Instructions::PushStack => {
            vm.stack_states.push(vm.stack_index as stack_index_type);
        }
        Instructions::PopStack => {
            vm.stack_index = match vm.stack_states.pop() {
                Some(previous_index) => previous_index as stack_pointer_type,
                None => return Err(InterpretError::InvalidStackPop),
            };
        }
        Instructions::DefineLocal => {


            let register = vm.read_register();

            let register_v = std::mem::replace(&mut vm.registers[register as usize], Value::Null);
            vm.define_local(register_v)?;

        }
        Instructions::GetLocal => {
            let output_register = vm.read_register();

            let index = vm.read_stack_index();




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
            let output_register = vm.read_register();


            let index = vm.read_stack_index();

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
            let register = vm.read_register();
            let jmp_addr = instruction_length_type::read_bytes(vm);

            if !vm.registers[register as usize].is_truthy() {
                vm.ip = jmp_addr as usize;
            }
        }
        Instructions::Jump => {
            let jmp_addr = instruction_length_type::read_bytes(vm);

            vm.ip = jmp_addr as usize;
        }
        Instructions::FunctionCall => {
            let fn_register = vm.read_register();

            let num_args = chunk.code[vm.ip];
            vm.ip += 1;

            let func_val = &vm.registers[fn_register as usize].inner();

            let mut call_closure = |c: &Closure<String>| {
                if c.function.arguments_count != num_args as u8 {
                    return Err(EvaluateErrorDetails::InvalidArgCount);
                }

                let offset = if c.function.function_kind == FunctionKind::Method {
                    1
                } else {
                    0
                };

                // Push a call frame
                vm.call_stack.push(CallFrame {
                    chunk: vm.current_chunk.clone(),
                    closure: c.clone(),
                    return_ip: vm.ip,
                    register_base: fn_register,
                    stack_index: vm.stack_index as stack_index_type - (num_args as stack_index_type + offset),
                    stack_state_index: vm.stack_states.len()
                });

                vm.stack_states.push(vm.stack_index as stack_index_type);

                // switch to closure's chunk
                vm.current_chunk = c.function.chunk.clone();
                vm.ip = 0;

                return Ok(())
            };

            match func_val {

                Value::Closure(c) => {
                    match c  {
                        crate::value::callable::Callable::LoxFunction(closure) => call_closure(&closure)?,
                        crate::value::callable::Callable::BindedLoxFunction(_, closure) => call_closure(closure)?,
                    };
                }


                Value::GlobalFunction(gf) => {
                    if let Some(arg_count) = gf.arguments_count {
                        if arg_count != num_args {
                            return Err(EvaluateErrorDetails::InvalidArgCount);
                        }
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
                    vm.stack_index -= num_args as stack_pointer_type;
                },
                Value::Class(c) => {
                    match c.constructor() {
                        Some(callable) => {
                            match callable {
                                Callable::LoxFunction(closure) => call_closure(&closure)?,
                                Callable::BindedLoxFunction(_, closure) => call_closure(&closure)?,
                            }
                        },
                        None => {
                            let class_instance = ClassInstance::new(c.clone());
                            vm.registers[fn_register as usize] = Value::Instance(class_instance);


                        },
                    }
                }

                _ => return Err(EvaluateErrorDetails::ExpectedFunction),
            }
        },
        Instructions::FunctionReturn => {
            let return_val = {
                let v = vm.call_stack.last().ok_or(EvaluateErrorDetails::InvalidReturnStatement)?;
                if v.closure.function.name == "init" && v.closure.function.function_kind == FunctionKind::Method {
                    let index = vm.get_stack_index(0) as usize;
                    std::mem::take(&mut vm.stack[index as usize])
                }
                else {
                    std::mem::take(&mut vm.registers[vm.get_register(0) as usize])
                }
            };
            let v = vm.call_stack.pop().ok_or(EvaluateErrorDetails::InvalidReturnStatement)?;
            vm.ip = v.return_ip;
            vm.current_chunk = v.chunk;



            vm.registers[v.register_base as usize] = return_val;
            vm.stack_index = v.stack_index as stack_pointer_type;

            vm.stack_states.truncate(v.stack_state_index as usize);

        },
        Instructions::DebugBreak => {
            for (i, v) in vm.registers.iter().enumerate().filter(|(_, i)| !i.is_null()) {
                println!("r{i} = {v}");
            }

            std::io::stdin().read_line(&mut String::new()).map_err(|_| EvaluateErrorDetails::StdinFailed)?;
        },
        Instructions::Closure => {
            let dst_register = vm.read_register();
            let constant = vm.read_constant();

            let func = &chunk.constants[constant as usize];

            let upvalue_count = chunk.code[vm.ip];
            vm.ip += 1;

            let mut upvalues = vec![];
            for _ in 0..upvalue_count {
                let is_local = chunk.code[vm.ip] != 0;
                vm.ip += 1;
                let index = vm.read_stack_index();

                if is_local {
                    let index = vm.get_stack_index(index) as usize;

                    vm.stack[index as usize] = vm.stack[index as usize].to_cell();
                    upvalues.push(vm.stack[index as usize].clone());
                } else {
                    upvalues.push(vm.call_stack.last().unwrap().closure.upvalues[index as usize].clone());
                }
            }

            let func = func.as_function()?;
            vm.registers[dst_register as usize] = Value::Closure(crate::value::callable::Callable::LoxFunction(Closure {
                function: func,
                upvalues,
            }));
        },
        Instructions::GetUpvalue => {
            let output_register = vm.read_register();

            let upvalue_index = vm.read_stack_index();

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
            let input_register = vm.read_register();

            let upvalue_index = vm.read_stack_index();

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
        },
        Instructions::GetField => {
            let register = vm.read_register();
            let constant = vm.read_constant();

            let v = &chunk.constants[constant as usize];
            vm.registers[register as usize] = match v {
                Value::String(a) => {
                    match &vm.registers[register as usize] {
                        Value::Instance(class_instance) => {
                            class_instance.get_field(a)?
                        }
                        _ => return Err(InterpretError::InvalidIdentifierType)
                    }
                },
                _ => {
                    return Err(InterpretError::InvalidIdentifierType);
                }
            };
        },
        Instructions::SetField => {
            let value_register = vm.read_register();
            let dist_register = vm.read_register();

            let constant = vm.read_constant();

            let v = &chunk.constants[constant as usize];
            match v {
                Value::String(a) => {
                    let mut instance = vm.registers[dist_register as usize].as_class_instance()?;
                    instance.set_field(a.clone(), vm.registers[value_register as usize].clone());
                },
                _ => {
                    return Err(InterpretError::InvalidIdentifierType);
                }
            };
        },
        Instructions::CreateMethod => {
            let value_register = vm.read_register();
            let dist_register = vm.read_register();

            match (vm.registers[value_register as usize].clone(), &mut vm.registers[dist_register as usize]) {
                (Value::Closure(closure), Value::Class(class)) => {
                    match &closure {
                        Callable::LoxFunction(c) => if c.function.name == "init".to_string() {
                            class.set_constructor(closure.clone());
                        },
                        Callable::BindedLoxFunction(_, _) => unreachable!(),
                    }

                    class.add_method(closure.clone());
                },
                _ => {
                    return Err(InterpretError::InvalidIdentifierType);
                }
            }
        },
        Instructions::InitFunction => {
            let dist_register = vm.read_register();
            match &vm.registers[dist_register as usize].inner() {
                Value::Closure(callable) => {
                    match callable {
                        crate::value::callable::Callable::LoxFunction(closure) => {
                            if closure.function.function_kind == FunctionKind::Method {
                                return Err(InterpretError::UnbindedMethod);
                            }
                        },
                        crate::value::callable::Callable::BindedLoxFunction(class_instance, _) =>  {
                            vm.define_local(Value::Instance(class_instance.clone()))?;
                        },
                    }
                },
                Value::Class(class) => {

                    if class.constructor().is_some() {
                        let class_instance = ClassInstance::new(class.clone());
                        vm.define_local(Value::Instance(class_instance.clone()))?;
                    }
                }
                Value::GlobalFunction(_) => {},

                a => {
                    return Err(InterpretError::InvalidIdentifierType)
                }

            }
        },
    }

    Ok(())
}

pub fn interpret_with_vm(vm: &mut Vm)  -> Result<(), EvaluateError> {
    let mut previous_ip = 0;
    while vm.ip < vm.current_chunk.code.len() {
        let instruction = vm.current_chunk.code[vm.ip];
        if DEBUG_TRACE_EXECUTION {
            let tmp = vm.ip;
            disassemble_instruction(Rc::clone(&vm.current_chunk), vm.ip, previous_ip);
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
                    line: vm.current_chunk.get_line(previous_ip as usize) as line_type,
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
