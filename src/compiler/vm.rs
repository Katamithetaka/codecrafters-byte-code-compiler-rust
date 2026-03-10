use std::cell::RefCell;

use crate::{
    compiler::{
        chunk::Chunk,
        garbage_collector::{FunctionKind, GcClosure, Heap, HeapObject, ResolvedObject},
        instructions::{Instructions, disassemble_instruction},
        int_types::{global_index_type, instruction_length_type, line_type, register_index_type, stack_index_type, stack_pointer_type, varint_type},
        varint::Varint,
    },
    expressions::{EvaluateError, EvaluateErrorDetails},
};

pub use crate::compiler::int_types::VmRead;

pub use crate::expressions::Value;

const STACK_MAX_SIZE: stack_pointer_type = stack_index_type::MAX as stack_pointer_type;
const REGISTER_MAX_SIZE: usize = 256;
const DEBUG_TRACE_EXECUTION: bool = false;
type Registers = [Value; REGISTER_MAX_SIZE];

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_ip: usize,
    pub previous_register_base: register_index_type,
    pub previous_stack_index: stack_index_type,
    pub stack_state_index: usize,
    pub chunk: &'static Chunk,
    pub closure: GcClosure,
    pub is_constructor: bool,
    pub args_count: usize,
}

pub struct VmData {
    pub ip: usize,
    pub registers: Registers,
    pub register_base: register_index_type,
    pub stack_base: stack_index_type,
    pub global_variables: Vec<Option<Value>>,
    pub stack_states: Vec<stack_index_type>,
    pub stack: Vec<Value>,
    pub stack_index: stack_pointer_type,
    pub call_stack: Vec<CallFrame>,
    pub current_chunk: &'static Chunk,
}

pub struct Vm {
    pub core: VmData,
    pub heap: Heap

}

impl Vm {
    pub fn get_register(&self, register: register_index_type) -> register_index_type {
        let frame_base = self.core.register_base;
        frame_base + register
    }

    pub fn get_stack_index(&self, stack_index: stack_index_type) -> stack_index_type {
        let frame_base = self.core.stack_base;
        frame_base + stack_index
    }

    pub fn read_register(&mut self) -> register_index_type {
        let register = register_index_type::read_bytes(self);
        return self.get_register(register);
    }

    pub fn read_constant(&mut self) -> varint_type {
        let (constant, size) = Varint::read_bytes(&self.core.current_chunk, self.core.ip);
        self.core.ip += size;
        return constant;
    }

    pub fn read_stack_index(&mut self) -> stack_index_type {
        return stack_index_type::read_bytes(self);
    }

    pub fn read_global(&mut self) -> global_index_type {
        return global_index_type::read_bytes(self);
    }

    pub fn define_local(&mut self, value: Value) -> Result<(), InterpretError> {
        if DEBUG_TRACE_EXECUTION {
            eprintln!("Writing local {}", self.core.stack_index);
        }

        self.core.stack[self.core.stack_index as usize] = value;

        self.core.stack_index += 1;

        if (self.core.stack_index) >= STACK_MAX_SIZE {
            self.core.stack_index;
            Err(InterpretError::StackOverflow)?;
        }

        Ok(())
    }

    fn exec_constant(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;

        let value = self.read_constant();
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", self.heap.to_string(chunk.constants[value as usize]));
        }
        Ok(())
    }

    fn exec_load(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;
        let register = self.read_register();
        let constant = self.read_constant();
        self.core.registers[register as usize] = chunk.constants[constant as usize].clone();
        Ok(())
    }

    fn read_value(&self, register: usize) -> Value {
        match self.core.registers[register] {
            Value::Cell(gc) => match self.heap.get(gc) {
                HeapObject::Cell(inner) => *inner.borrow(),
                _ => panic!("expected cell"),
            },
            other => other,
        }
    }

    fn value_as_number(&self, value: Value) -> Result<f64, InterpretError> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(EvaluateErrorDetails::ExpectedNumber),
        }
    }

    fn value_is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Null => false,
            Value::Boolean(b) => b,
            _ => true,
        }
    }


    fn exec_negate(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let dst_register = self.read_register();
        let v = self.value_as_number(self.read_value(register as usize))?;
        self.core.registers[dst_register as usize] = Value::Number(-v);
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]));
        }
        Ok(())
    }

    fn exec_bang(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let dst_register = self.read_register();
        self.core.registers[dst_register as usize] =
            Value::Boolean(!self.value_is_truthy(self.read_value(register as usize)));
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]));
        }
        Ok(())
    }

    fn exec_binary_number_op<F>(&mut self, f: F) -> Result<(), InterpretError>
    where F: Fn(f64, f64) -> Value {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        let a = self.value_as_number(self.read_value(register_0 as usize))?;
        let b = self.value_as_number(self.read_value(register_1 as usize))?;
        self.core.registers[dst_register as usize] = f(a, b);
        Ok(())
    }

    fn exec_add(&mut self) -> Result<(), InterpretError> {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        let v_0 = self.read_value(register_0 as usize);
        let v_1 = self.read_value(register_1 as usize);
        self.core.registers[dst_register as usize] = match (v_0, v_1) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => {
                let sa = match self.heap.get(a) { HeapObject::String(s) => s.clone(), _ => panic!("expected string") };
                let sb = match self.heap.get(b) { HeapObject::String(s) => s.clone(), _ => panic!("expected string") };
                Value::String(self.heap.alloc(HeapObject::String(format!("{sa}{sb}"))))
            }
            _ => Err(InterpretError::UnmatchedTypes)?,
        };
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]));
        }
        Ok(())
    }

    fn exec_sub(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Number(a - b))
    }

    fn exec_mul(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Number(a * b))

    }

    fn exec_div(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Number(a / b))

    }

    fn exec_eq(&mut self) -> Result<(), InterpretError> {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        self.core.registers[dst_register as usize] =
            Value::Boolean(self.heap.equals(self.read_value(register_0 as usize), self.read_value(register_1 as usize)));
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]));
        }
        Ok(())
    }

    fn exec_neq(&mut self) -> Result<(), InterpretError> {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        self.core.registers[dst_register as usize] =
            Value::Boolean(!self.heap.equals(self.read_value(register_0 as usize), self.read_value(register_1 as usize)));
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]));
        }
        Ok(())
    }

    fn exec_lt(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Boolean(a < b))

    }

    fn exec_gt(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Boolean(a > b))

    }

    fn exec_lteq(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Boolean(a <= b))

    }

    fn exec_gteq(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::Boolean(a >= b))

    }

    fn exec_print(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        println!("{}", self.heap.to_string(self.read_value(register as usize)));
        Ok(())
    }

    fn exec_define_global(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let constant = self.read_global();

        let register_v = self.core.registers[register as usize];
        *self.core.global_variables.get_mut(constant as usize)
            .ok_or_else(|| InterpretError::UndefinedVariable(constant.to_string()))? = Some(register_v);
        Ok(())
    }

    fn exec_get_global(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let constant = self.read_global();

        let value = self.core.global_variables[constant as usize].ok_or_else(|| InterpretError::UndefinedVariable(constant.to_string()))?;

        self.core.registers[register as usize] = value;
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[register as usize]));
        }
        Ok(())
    }

    fn exec_set_global(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let constant = self.read_global();

        let variable = self.core
            .global_variables
            .get_mut(constant as usize)
            .ok_or_else(|| InterpretError::UndefinedVariable(constant.to_string()))?
            .as_mut()
            .ok_or_else(|| InterpretError::UndefinedVariable(constant.to_string()))?;



        *variable = self.core.registers[register as usize].clone();
        Ok(())
    }

    fn exec_push_stack(&mut self) -> Result<(), InterpretError> {
        self.core.stack_states.push(self.core.stack_index as stack_index_type);
        Ok(())
    }

    fn exec_pop_stack(&mut self) -> Result<(), InterpretError> {
        let previous_index = self.core.stack_states.pop().ok_or_else(|| InterpretError::InvalidStackPop)?;
        self.core.stack_index = previous_index as stack_pointer_type;
        Ok(())
    }

    fn exec_define_local(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let register_v = self.core.registers[register as usize];
        self.define_local(register_v)?;
        Ok(())
    }

    fn exec_get_local(&mut self) -> Result<(), InterpretError> {
        let output_register = self.read_register();
        let index = self.read_stack_index();
        let index = self.get_stack_index(index) as usize;

        if DEBUG_TRACE_EXECUTION {
            eprintln!("Getting local {}", index);
        }

        if index >= self.core.stack_index as usize {
            Err(EvaluateErrorDetails::UndefinedVariable(format!(
                "Stack {index}: Index is too high",
            )))?;
        }

        self.core.registers[output_register as usize] = self.core.stack[index as usize].clone();
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[output_register as usize]));
        }
        Ok(())
    }

    fn exec_set_local(&mut self) -> Result<(), InterpretError> {
        let output_register = self.read_register();
        let index = self.read_stack_index();
        let index = self.get_stack_index(index) as usize;

        if DEBUG_TRACE_EXECUTION {
            eprintln!("Getting local {index}");
        }

        if index >= self.core.stack_index as usize {
            Err(EvaluateErrorDetails::UndefinedVariable(format!(
                "Stack {index}: Index is too high",
            )))?;
        }

        self.heap.set(&mut self.core.stack[index as usize], self.core.registers[output_register as usize]);
        Ok(())
    }

    fn exec_jump_if_false(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let jmp_addr = instruction_length_type::read_bytes(self);

        if !self.value_is_truthy(self.read_value(register as usize)) {
            self.core.ip = jmp_addr as usize;
        }
        Ok(())
    }

    fn exec_jump(&mut self) -> Result<(), InterpretError> {
        let jmp_addr = instruction_length_type::read_bytes(self);
        self.core.ip = jmp_addr as usize;
        Ok(())
    }

    fn call_closure(&mut self, fn_register: register_index_type, num_args: u8, c: GcClosure, is_constructor: bool) -> Result<(), InterpretError> {

        let func_gc = match c.function {
            Value::Function(gc) => gc,
            _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
        };
        let func = match self.heap.get(func_gc) {
            HeapObject::Function(f) => f,
            _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
        };

        if func.arguments_count != num_args {
            Err(EvaluateErrorDetails::InvalidArgCount)?;
        }

        let offset = match func.function_kind {
            FunctionKind::Method { is_derived: true } => {

                2
            },
            FunctionKind::Method { is_derived: false } => 1,
            FunctionKind::Function => 0,
        };

        let chunk = func.chunk;
        self.core.call_stack.push(CallFrame {
            chunk: self.core.current_chunk,
            closure: c,
            return_ip: self.core.ip,
            previous_register_base: self.core.register_base,
            previous_stack_index: self.core.stack_base,
            stack_state_index: self.core.stack_states.len(),
            is_constructor,
            args_count: num_args as usize,
        });

        self.core.stack_states.push(self.core.stack_index as stack_index_type);
        self.core.register_base = fn_register;
        self.core.stack_base = self.core.stack_index as stack_index_type - (num_args as stack_index_type + offset);
        self.core.current_chunk = chunk;
        self.core.ip = 0;

        Ok(())
    }

    fn exec_function_call(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;
        let fn_register = self.read_register();

        let num_args = chunk.code[self.core.ip];
        self.core.ip += 1;

        let fn_val = self.read_value(fn_register as usize);
        self.exec_init_function(fn_val)?;

        match fn_val {
            Value::Closure(gc) => {
                let c = match self.heap.get(gc) {
                    HeapObject::Closure(c) => *c,
                    _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                };

                let f_gc = match c.function {
                    Value::Function(gc) => gc,
                    _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                };

                let f = match self.heap.get(f_gc) {
                    HeapObject::Function(c) => c,
                    _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                };


                let is_constructor = if let FunctionKind::Method { .. } = f.function_kind {
                    let f_gc = match f.name {
                        Value::String(c) => c,
                        _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                    };


                    let f_name = match self.heap.get(f_gc) {
                        HeapObject::String(s) => s,
                        _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                    };

                    f_name == "init"
                }
                else {
                    false
                };


                self.call_closure(fn_register, num_args, c, is_constructor)?;
            }

            Value::GlobalFunction(gc) => {
                let (arg_count, callable) = match self.heap.get(gc) {
                    HeapObject::GlobalFunction(gf) => (gf.arguments_count, gf.callable.clone()),
                    _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                };
                if let Some(expected) = arg_count {
                    if expected != num_args {
                        Err(EvaluateErrorDetails::InvalidArgCount)?;
                    }
                }

                let args_start = self.core.stack_index as usize - num_args as usize;
                let args: Vec<_> = if args_start < self.core.stack_index as usize {
                    let slice = &mut self.core.stack[args_start..self.core.stack_index as usize];
                    slice.iter_mut().map(std::mem::take).collect()
                } else {
                    vec![]
                };

                let return_val = (callable)(args);
                self.core.registers[fn_register as usize] = return_val;
                self.core.stack_index -= num_args as stack_pointer_type;
            }

            Value::Class(gc) => {
                let class_val = fn_val;
                let constructor = match self.heap.get(gc) {
                    HeapObject::Class(c) => c.constructor,
                    _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                };
                match constructor {
                    Some(ctor_val) => {
                        let closure = match ctor_val {
                            Value::Closure(cgc) => match self.heap.get(cgc) {
                                HeapObject::Closure(c) => *c,
                                _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                            },
                            _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
                        };
                        self.call_closure(fn_register, num_args, closure, true)?;
                    }
                    None => {
                        self.core.registers[fn_register as usize] = self.heap.instance_create(class_val);
                    }
                }
            }

            _ => Err(EvaluateErrorDetails::ExpectedFunction)?,
        }

        Ok(())
    }

    fn exec_function_return(&mut self) -> Result<(), InterpretError> {
        let return_val = {
            let v = self.core.call_stack.last()    .ok_or_else(|| EvaluateErrorDetails::InvalidReturnStatement)?;



            if v.is_constructor {
                let index = self.get_stack_index(v.args_count as stack_index_type) as usize;
                std::mem::take(&mut self.core.stack[index as usize])
            } else {
                std::mem::take(&mut self.core.registers[self.get_register(0) as usize])
            }
        };

        let v = self.core.call_stack.pop().ok_or_else(|| EvaluateErrorDetails::InvalidReturnStatement)?;
        self.core.ip = v.return_ip;
        self.core.current_chunk = v.chunk;

        self.core.registers[self.core.register_base as usize] = return_val;
        self.core.register_base = v.previous_register_base;
        self.core.stack_index = self.core.stack_base as stack_pointer_type;
        self.core.stack_base = v.previous_stack_index;

        self.core.stack_states.truncate(v.stack_state_index as usize);

        Ok(())
    }

    fn exec_debug_break(&mut self) -> Result<(), InterpretError> {

        std::io::stdin().read_line(&mut String::new()).map_err(|_| EvaluateErrorDetails::StdinFailed)?;
        Ok(())
    }

    fn exec_closure(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;
        let dst_register = self.read_register();
        let constant = self.read_constant();

        let func = &chunk.constants[constant as usize];

        let upvalue_count = chunk.code[self.core.ip];
        self.core.ip += 1;

        let mut upvalues = vec![];
        for _ in 0..upvalue_count {
            let is_local = chunk.code[self.core.ip] != 0;
            self.core.ip += 1;
            let index = self.read_stack_index();

            if is_local {
                let index = self.get_stack_index(index) as usize;
                let cell = self.heap.alloc(HeapObject::Cell(RefCell::new(self.core.stack[index as usize])));
                self.core.stack[index as usize] = Value::Cell(cell);
                upvalues.push(self.core.stack[index as usize].clone());
            } else {
                match self.heap.get(self.core.call_stack.last().unwrap().closure.upvalues) {
                    HeapObject::ValueVec(v) => {
                        upvalues.push(v[index as usize])
                    },
                    _ => panic!("Upvalues wasn't a vector!")
                };
            }
        }

        let function_kind = match self.heap.resolve(*func) {
            ResolvedObject::Function(gc_function) => gc_function.function_kind,
            _ => unreachable!()
        };

        let upvalues = self.heap.alloc(HeapObject::ValueVec(upvalues));


        let closure = self.heap.alloc(HeapObject::Closure(GcClosure { class: None, instance: None, function: *func, upvalues, function_kind }));

        self.core.registers[dst_register as usize] = Value::Closure(closure);

        Ok(())
    }

    fn exec_get_upvalue(&mut self) -> Result<(), InterpretError> {
        let output_register = self.read_register();
        let upvalue_index = self.read_stack_index();

        let upvalues_gc = self.core.call_stack.last()
            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?
            .closure.upvalues;

        let raw = match self.heap.get(upvalues_gc) {
            HeapObject::ValueVec(v) => v[upvalue_index as usize],
            _ => panic!("Upvalues wasn't a vector!"),
        };

        // If the upvalue is a Cell, unwrap it; otherwise use directly.
        self.core.registers[output_register as usize] = match raw {
            Value::Cell(gc) => match self.heap.get(gc) {
                HeapObject::Cell(cell) => *cell.borrow(),
                _ => panic!("expected cell"),
            },
            other => other,
        };

        Ok(())
    }

    fn exec_set_upvalue(&mut self) -> Result<(), InterpretError> {
        let input_register = self.read_register();
        let upvalue_index = self.read_stack_index();

        let upvalues_gc = self.core.call_stack.last()
            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?
            .closure.upvalues;

        let new_val = self.core.registers[input_register as usize];

        let cell_gc = match self.heap.get(upvalues_gc) {
            HeapObject::ValueVec(v) => match v[upvalue_index as usize] {
                Value::Cell(gc) => gc,
                _ => panic!("Upvalue isn't a Cell!"),
            },
            _ => panic!("Upvalues wasn't a vector!"),
        };

        match self.heap.get(cell_gc) {
            HeapObject::Cell(cell) => *cell.borrow_mut() = new_val,
            _ => panic!("expected cell"),
        }



        Ok(())
    }

    fn exec_get_field(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;
        let register = self.read_register();
        let constant = self.read_constant();

        let key_val = chunk.constants[constant as usize];
        let field_name = match self.heap.resolve_inner(key_val) {
            ResolvedObject::String(s) => s.to_string(),
            _ => Err(InterpretError::InvalidIdentifierType)?,
        };



        let instance_gc = match self.core.registers[register as usize] {
            Value::Instance(gc) => gc,
            _ => return Err(InterpretError::InvalidIdentifierType),
        };



        self.core.registers[register as usize] = self
            .heap
            .instance_get(instance_gc, &field_name)
            .ok_or_else(|| EvaluateErrorDetails::UndefinedVariable(field_name))?;

        Ok(())
    }

    fn exec_set_field(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;
        let value_register = self.read_register();
        let dist_register = self.read_register();
        let constant = self.read_constant();

        let key_val = chunk.constants[constant as usize];
        let field_name = match self.heap.resolve_inner(key_val) {
            ResolvedObject::String(s) => s.to_string(),
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        let instance_gc = match self.core.registers[dist_register as usize] {
            Value::Instance(gc) => gc,
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        let new_value = self.core.registers[value_register as usize];
        self.heap.instance_set_field(instance_gc, field_name, new_value);

        Ok(())
    }

    fn exec_create_method(&mut self) -> Result<(), InterpretError> {
        let value_register = self.read_register();
        let dist_register = self.read_register();

        let closure_val = self.core.registers[value_register as usize];
        let class_val = self.core.registers[dist_register as usize];

        let (closure_gc, class_gc) = match (closure_val, class_val) {
            (Value::Closure(c), Value::Class(k)) => (c, k),
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        // Stamp the owning class onto the closure
        match self.heap.get_mut(closure_gc) {
            HeapObject::Closure(c) => c.class = Some(class_val),
            _ => return Err(InterpretError::InvalidIdentifierType),
        }

        // Derive the method name from the underlying function
        let func_val = match self.heap.get(closure_gc) {
            HeapObject::Closure(c) => c.function,
            _ => return Err(InterpretError::InvalidIdentifierType),
        };
        let method_name_str = match self.heap.resolve_inner(func_val) {
            ResolvedObject::Function(f) => self.heap.to_string(f.name),
            _ => return Err(InterpretError::InvalidIdentifierType),
        };
        let method_name_val = self.heap.alloc_string(method_name_str.clone());

        self.heap.class_add_method(class_gc, method_name_val, Value::Closure(closure_gc));

        Ok(())
    }

    fn exec_init_function(&mut self, value: Value) -> Result<(), InterpretError> {


        match value {
            Value::Closure(closure_gc) => {
                let c = match self.heap.get(closure_gc) {
                    HeapObject::Closure(c) => *c,
                    _ => return Err(InterpretError::InvalidIdentifierType),
                };

                let func_kind = c.function_kind;

                match func_kind {
                    FunctionKind::Function => return Ok(()),
                    _ => {}
                }

                let func_gc = match c.function {
                    Value::Function(gc) => gc,
                    _ => return Err(InterpretError::InvalidIdentifierType),
                };
                let class_val = match self.heap.get(func_gc) {
                    HeapObject::Function(_) => c.class,
                    _ => return Err(InterpretError::InvalidIdentifierType),
                };
                match func_kind {
                    FunctionKind::Function => {
                        // plain function — nothing to push onto the stack
                    }
                    FunctionKind::Method { is_derived } => {
                        let instance = c.instance.ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;
                        self.define_local(instance)?;
                        if is_derived {
                            let class = class_val.ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;
                            self.define_local(class)?;
                            if DEBUG_TRACE_EXECUTION {
                                eprintln!("Defining super!");
                            }
                        }
                    }
                }
            }
            Value::Class(class_gc) => {
                let class_val = value;
                let constructor = match self.heap.get(class_gc) {
                    HeapObject::Class(c) => c.constructor,
                    _ => return Err(InterpretError::InvalidIdentifierType),
                };
                if let Some(ctor_val) = constructor {
                    let instance_val = self.heap.instance_create(class_val);
                    self.define_local(instance_val)?;

                    let is_derived = match ctor_val {
                        Value::Closure(cgc) => match self.heap.get(cgc) {
                            HeapObject::Closure(c) => match c.function {
                                Value::Function(fgc) => match self.heap.get(fgc) {
                                    HeapObject::Function(f) => matches!(f.function_kind, FunctionKind::Method { is_derived: true }),
                                    _ => false,
                                },
                                _ => false,
                            },
                            _ => false,
                        },
                        _ => false,
                    };
                    if is_derived {
                        let base = match self.heap.get(class_gc) {
                            HeapObject::Class(c) => c.base_class.ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?,
                            _ => unreachable!(),
                        };
                        self.define_local(base)?;
                        if DEBUG_TRACE_EXECUTION {
                            eprintln!("Defining super!");
                        }
                    }
                }
            }
            Value::GlobalFunction(_) => {}
            _ => return Err(InterpretError::InvalidIdentifierType),
        }

        Ok(())
    }

    fn exec_set_base_class(&mut self) -> Result<(), InterpretError> {
        let value_register = self.read_register();
        let dist_register = self.read_register();

        let src_val = std::mem::replace(&mut self.core.registers[value_register as usize], Value::Null);
        let dst_val = self.core.registers[dist_register as usize];

        let (src_gc, dst_gc) = match (src_val, dst_val) {
            (Value::Class(s), Value::Class(d)) => (s, d),
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        self.heap.class_inherit(dst_gc, src_gc);

        Ok(())
    }

    fn exec_super(&mut self) -> Result<(), InterpretError> {
        let value_register = self.read_register();
        let this_register = self.read_register();
        let super_register = self.read_register();
        let dist_register = self.read_register();

        // `value_register` holds a string naming the method we want
        let identifier = match self.read_value(value_register as usize) {
            Value::String(gc) => match self.heap.get(gc) {
                HeapObject::String(s) => s.clone(),
                _ => return Err(InterpretError::InvalidIdentifierType),
            },
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        // `super_register` holds the class value
        let class_gc = match self.read_value(super_register as usize) {
            Value::Class(gc) => gc,
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        let base_class_gc = match self.heap.get(class_gc) {
            HeapObject::Class(c) => match c.base_class.ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)? {
                Value::Class(gc) => gc,
                _ => return Err(InterpretError::InvalidIdentifierType),
            },
            _ => return Err(InterpretError::InvalidIdentifierType),
        };

        let method_val = self
            .heap
            .class_get_method(base_class_gc, &identifier)
            .ok_or_else(|| EvaluateErrorDetails::UndefinedVariable(identifier))?;

        let instance_val = self.read_value(this_register as usize);
        self.core.registers[dist_register as usize] = self.heap.bind_method(method_val, instance_val);

        Ok(())
    }
}

pub fn save_registers(registers: &mut Registers) -> Registers {
    std::mem::replace(registers, std::array::from_fn(|_| Value::Null))
}

impl Vm {
    pub fn new(chunk: &'static Chunk, globals_count: global_index_type) -> Self {
        Self {
            core: VmData {
                register_base: 0,
                ip: 0,
                registers: std::array::from_fn(|_| Value::Null),
                global_variables: vec![None; globals_count as usize],
                stack_states: Vec::with_capacity(255),
                stack: vec![Value::Null; STACK_MAX_SIZE as usize],
                stack_index: 0,
                call_stack: Vec::with_capacity(255),
                current_chunk: chunk,
                stack_base: 0,
            },
            heap: Heap::new()
        }
    }
}

pub type InterpretError = crate::expressions::EvaluateErrorDetails;

#[inline(always)]
pub fn execute_instruction(
    vm: &mut Vm,
    instruction: Instructions,
) -> Result<(), InterpretError> {
    match instruction {
        Instructions::Return => Ok(()),
        Instructions::Constant => vm.exec_constant(),
        Instructions::Load => vm.exec_load(),
        Instructions::Negate => vm.exec_negate(),
        Instructions::Bang => vm.exec_bang(),
        Instructions::Add => vm.exec_add(),
        Instructions::Sub => vm.exec_sub(),
        Instructions::Mul => vm.exec_mul(),
        Instructions::Div => vm.exec_div(),
        Instructions::Eq => vm.exec_eq(),
        Instructions::Neq => vm.exec_neq(),
        Instructions::Lt => vm.exec_lt(),
        Instructions::Gt => vm.exec_gt(),
        Instructions::LtEq => vm.exec_lteq(),
        Instructions::GtEq => vm.exec_gteq(),
        Instructions::Print => vm.exec_print(),
        Instructions::DefineGlobal => vm.exec_define_global(),
        Instructions::GetGlobal => vm.exec_get_global(),
        Instructions::SetGlobal => vm.exec_set_global(),
        Instructions::PushStack => vm.exec_push_stack(),
        Instructions::PopStack => vm.exec_pop_stack(),
        Instructions::DefineLocal => vm.exec_define_local(),
        Instructions::GetLocal => vm.exec_get_local(),
        Instructions::SetLocal => vm.exec_set_local(),
        Instructions::JumpIfFalse => vm.exec_jump_if_false(),
        Instructions::Jump => vm.exec_jump(),
        Instructions::FunctionCall => vm.exec_function_call(),
        Instructions::FunctionReturn => vm.exec_function_return(),
        Instructions::DebugBreak => vm.exec_debug_break(),
        Instructions::Closure => vm.exec_closure(),
        Instructions::GetUpvalue => vm.exec_get_upvalue(),
        Instructions::SetUpvalue => vm.exec_set_upvalue(),
        Instructions::GetField => vm.exec_get_field(),
        Instructions::SetField => vm.exec_set_field(),
        Instructions::CreateMethod => vm.exec_create_method(),
        Instructions::SetBaseClass => vm.exec_set_base_class(),
        Instructions::Super => vm.exec_super(),
    }
}

pub fn interpret_with_vm(vm: &mut Vm) -> Result<(), EvaluateError> {
    let mut previous_ip = 0;
    while vm.core.ip < vm.core.current_chunk.code.len() {
        let instruction = vm.core.current_chunk.code[vm.core.ip];
        if DEBUG_TRACE_EXECUTION {
            let tmp = vm.core.ip;
            disassemble_instruction(&vm.heap, vm.core.current_chunk, vm.core.ip, previous_ip);
            previous_ip = tmp;
        }

        vm.core.ip += 1;
        let instruction: Instructions = unsafe { std::mem::transmute(instruction) };
        execute_instruction(vm, instruction).or_else(|err| Err(EvaluateError {
            error: err,
            line: vm.core.current_chunk.get_line(previous_ip as usize) as line_type,
        }))?;
    }

    return Ok(());
}

pub fn interpret(chunk: &'static Chunk, global_count: global_index_type, heap: Heap) -> Result<(), EvaluateError> {
    eprintln!("{global_count}");
    let mut vm = Box::new(Vm::new(chunk, global_count));
    vm.heap = heap;
    interpret_with_vm(&mut vm)
}
