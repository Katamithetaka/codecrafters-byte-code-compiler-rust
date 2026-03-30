use std::{cell::RefCell, collections::LinkedList};

use arrayvec::ArrayVec;

use crate::{
    compiler::{
        chunk::Chunk,
        garbage_collector::{FunctionKind, Gc, GcClosure, Heap, HeapObject, ResolvedObject},
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
const MAX_STACK_DEPTH: usize = 256;
type Registers = [Value; REGISTER_MAX_SIZE];

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_ip: usize,
    pub stack_state_index: usize,
    pub args_count: usize,
    pub chunk: &'static Chunk,
    pub upvalues: Gc,
    pub previous_register_base: register_index_type,
    pub previous_stack_index: stack_index_type,
    pub is_constructor: bool,
}

pub struct VmData {
    pub ip: usize,
    pub current_chunk: &'static Chunk,

    pub call_stack: ArrayVec<CallFrame, MAX_STACK_DEPTH>,
    pub stack_states: ArrayVec<stack_index_type, MAX_STACK_DEPTH>,
    pub registers: Registers,

    pub global_variables: Vec<Option<Value>>,
    pub stack: Vec<Value>,

    pub stack_index: stack_pointer_type,
    pub register_base: register_index_type,
    pub stack_base: stack_index_type,

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
            eprintln!("{}", self.heap.to_string(chunk.constants[value as usize]).unwrap());
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

    fn read_value(&self, register: usize) -> Option<Value> {
        match self.core.registers[register] {
            v if v.is_cell() => self.heap.copy_value(self.core.registers[register]),
            other => Some(other),
        }
    }

    fn value_as_number(&self, value: Value) -> Result<f64, InterpretError> {
        value.is_number().then_some(value.as_number()).ok_or_else(|| EvaluateErrorDetails::ExpectedNumber)
    }

    fn value_is_truthy(&self, value: Value) -> bool {
        if value.is_null() {
            false
        }
        else if value.is_bool() {
            value.as_bool()
        }
        else {
            true
        }


    }


    fn exec_negate(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let dst_register = self.read_register();
        let v = self.value_as_number(self.read_value(register as usize).unwrap())?;
        self.core.registers[dst_register as usize] = Value::number(-v);
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]).unwrap());
        }
        Ok(())
    }

    fn exec_bang(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        let dst_register = self.read_register();
        self.core.registers[dst_register as usize] =
            Value::bool(!self.value_is_truthy(self.read_value(register as usize).unwrap()));
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]).unwrap());
        }
        Ok(())
    }

    fn exec_binary_number_op<F>(&mut self, f: F) -> Result<(), InterpretError>
    where F: Fn(f64, f64) -> Value {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        let a = self.value_as_number(self.read_value(register_0 as usize).unwrap())?;
        let b = self.value_as_number(self.read_value(register_1 as usize).unwrap())?;
        self.core.registers[dst_register as usize] = f(a, b);
        Ok(())
    }

    fn exec_add(&mut self) -> Result<(), InterpretError> {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        let v_0 = self.read_value(register_0 as usize).unwrap();
        let v_1 = self.read_value(register_1 as usize).unwrap();
        self.core.registers[dst_register as usize] = match (v_0, v_1) {
            _ if v_0.is_number() && v_1.is_number() => Value::number(v_0.as_number() + v_1.as_number()),
            _ if v_0.is_string() && v_1.is_string() => {
                Value::string(self.heap.alloc(HeapObject::String(format!("{}{}",
                    self.heap.resolve_string(v_0.unwrap_gc()).unwrap(),
                    self.heap.resolve_string(v_1.unwrap_gc()).unwrap()
                ))))
            }
            _ => Err(InterpretError::UnmatchedTypes)?,
        };
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]).unwrap());
        }
        Ok(())
    }

    fn exec_sub(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::number(a - b))
    }

    fn exec_mul(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::number(a * b))

    }

    fn exec_div(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::number(a / b))

    }

    fn exec_eq(&mut self) -> Result<(), InterpretError> {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        self.core.registers[dst_register as usize] =
            Value::bool(self.heap.equals(self.read_value(register_0 as usize).unwrap(), self.read_value(register_1 as usize).unwrap()));
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]).unwrap());
        }
        Ok(())
    }

    fn exec_neq(&mut self) -> Result<(), InterpretError> {
        let register_0 = self.read_register();
        let register_1 = self.read_register();
        let dst_register = self.read_register();
        self.core.registers[dst_register as usize] =
            Value::bool(!self.heap.equals(self.read_value(register_0 as usize).unwrap(), self.read_value(register_1 as usize).unwrap()));
        if DEBUG_TRACE_EXECUTION {
            eprintln!("{}", &self.heap.to_string(self.core.registers[dst_register as usize]).unwrap());
        }
        Ok(())
    }

    fn exec_lt(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::bool(a < b))

    }

    fn exec_gt(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::bool(a > b))

    }

    fn exec_lteq(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::bool(a <= b))

    }

    fn exec_gteq(&mut self) -> Result<(), InterpretError> {
        self.exec_binary_number_op(|a, b| Value::bool(a >= b))

    }

    fn exec_print(&mut self) -> Result<(), InterpretError> {
        let register = self.read_register();
        println!("{}", self.heap.to_string(self.read_value(register as usize).unwrap()).unwrap());
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
            eprintln!("{}", &self.heap.to_string(self.core.registers[register as usize]).unwrap());
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
            eprintln!("{}", &self.heap.to_string(self.core.registers[output_register as usize]).unwrap());
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

        if !self.value_is_truthy(self.read_value(register as usize).unwrap()) {
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

        let func = self.heap.resolve_function(c.function).ok_or_else(|| {
            EvaluateErrorDetails::ExpectedFunction
        })?;

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
            upvalues: c.upvalues,
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

        let fn_val = self.read_value(fn_register as usize).unwrap();
        self.exec_init_function(fn_val)?;

        match fn_val {
            _ if fn_val.is_closure() => {
                let c = self.heap.resolve_closure(fn_val.unwrap_gc()).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

                let f = self.heap.resolve_function(c.function).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

                let is_constructor = if let FunctionKind::Method { .. } = f.function_kind {
                    let f_name = self.heap.resolve_string(f.name).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

                    f_name == "init"
                }
                else {
                    false
                };


                self.call_closure(fn_register, num_args, *c, is_constructor)?;
            }

            _ if fn_val.is_global_function() => {
                let (arg_count, callable) = self.heap.resolve_global_function(fn_val.unwrap_gc()).map(|gf| (gf.arguments_count, gf.callable.clone())).ok_or_else(||  EvaluateErrorDetails::ExpectedFunction)?;

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

            _ if fn_val.is_class() => {
                let class_val = fn_val;
                let constructor = self.heap.resolve_class(class_val.unwrap_gc()).map(|c| c.constructor).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

                match constructor.as_option() {
                    Some(ctor_val) => {
                        let closure = self.heap.resolve_closure(ctor_val).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;
                        self.call_closure(fn_register, num_args, *closure, true)?;
                    }
                    None => {
                        self.core.registers[fn_register as usize] = self.heap.instance_create(class_val.unwrap_gc()).unwrap();
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
                if !self.core.stack[index as usize].is_cell() {
                    let cell = self.heap.alloc(HeapObject::Cell(RefCell::new(self.core.stack[index as usize])));
                    self.core.stack[index as usize] = Value::cell(cell);
                }
                upvalues.push(self.core.stack[index as usize]);
            } else {
                let parent_upvalues = self.heap.resolve_value_vec(self.core.call_stack.last().unwrap().upvalues)            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?;

                upvalues.push(parent_upvalues[index as usize])
            }
        }

        let function_kind =
            func.is_function().then(|| self.heap.resolve_function(func.unwrap_gc()).map(|f| f.function_kind)).flatten().ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

        let upvalues = self.heap.alloc(HeapObject::ValueVec(upvalues));
        let closure = self.heap.alloc(HeapObject::Closure(GcClosure { class: Gc::NONE, instance: Gc::NONE, function: func.unwrap_gc(), upvalues, function_kind }));

        self.core.registers[dst_register as usize] = Value::closure(closure);

        Ok(())
    }

    fn exec_get_upvalue(&mut self) -> Result<(), InterpretError> {
        let output_register = self.read_register();
        let upvalue_index = self.read_stack_index();

        let upvalues_gc = self.core.call_stack.last()
            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?
            .upvalues;

        let raw = self.heap.resolve_value_vec(upvalues_gc)
            .map(|v| v[upvalue_index as usize])
            .and_then(|v| v.is_cell().then_some(v.unwrap_gc()))
            .and_then(|v| self.heap.resolve_cell(v))
            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?;

        self.core.registers[output_register as usize] = *raw.borrow();

        Ok(())
    }

    fn exec_set_upvalue(&mut self) -> Result<(), InterpretError> {
        let input_register = self.read_register();
        let upvalue_index = self.read_stack_index();

        let upvalues_gc = self.core.call_stack.last()
            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?
            .upvalues;

        let new_val = self.core.registers[input_register as usize];

        let raw = self.heap.resolve_value_vec(upvalues_gc)
            .map(|v| v[upvalue_index as usize])
            .and_then(|v| v.is_cell().then_some(v.unwrap_gc()))
            .and_then(|v| self.heap.resolve_cell(v))
            .ok_or_else(|| EvaluateErrorDetails::InvalidUpvalueAccess)?;

        *raw.borrow_mut() = new_val;


        Ok(())
    }

    fn exec_get_field(&mut self) -> Result<(), InterpretError> {
        let chunk = self.core.current_chunk;
        let register = self.read_register();
        let constant = self.read_constant();

        let key_val = chunk.constants[constant as usize];
        let field_name = self.heap.to_string(key_val).unwrap();


        let instance_gc = self.read_value(register as usize).and_then(|c|
            c.is_instance().then_some(c.unwrap_gc())
        ).ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;



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
        let field_name = self.heap.to_string(key_val).unwrap();


        let instance_gc = self.read_value(dist_register as usize).and_then(|c|
            c.is_instance().then_some(c.unwrap_gc())
        ).ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;


        let new_value = self.core.registers[value_register as usize];
        self.heap.instance_set_field(instance_gc, field_name, new_value);

        Ok(())
    }

    fn exec_create_method(&mut self) -> Result<(), InterpretError> {
        let value_register = self.read_register();
        let dist_register = self.read_register();

        let closure_gc = Some(self.core.registers[value_register as usize]).map(|c| c.is_closure().then(|| c.unwrap_gc().as_option()).flatten()).flatten().ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;
        let class_gc = Some(self.core.registers[dist_register as usize]).map(|c| c.is_class().then(|| c.unwrap_gc().as_option()).flatten()).flatten().ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;




        // Stamp the owning class onto the closure
        match self.heap.get_mut(closure_gc) {
            HeapObject::Closure(c) => c.class = class_gc,
            _ => return Err(InterpretError::InvalidIdentifierType),
        }

        // Derive the method name from the underlying function
        let func_val = self.heap.resolve_closure(closure_gc).map(|c| c.function).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

        let method_name_str = self.heap.resolve_function(func_val).and_then(|f| self.heap.resolve_string(f.name)).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;
        let method_name_val = self.heap.alloc_string(method_name_str.to_string());

        self.heap.class_add_method(class_gc, method_name_val, Value::closure(closure_gc));

        Ok(())
    }

    fn exec_init_function(&mut self, value: Value) -> Result<(), InterpretError> {


        match value {
            _ if value.is_closure() => {
                let c = self.heap.resolve_closure(value.unwrap_gc()).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

                let func_kind = c.function_kind;

                match func_kind {
                    FunctionKind::Function => return Ok(()),
                    _ => {}
                }

                let func_gc = c.function;
                let class_val = match self.heap.get(func_gc) {
                    HeapObject::Function(_) => c.class,
                    _ => return Err(InterpretError::InvalidIdentifierType),
                };
                match func_kind {
                    FunctionKind::Function => {
                        // plain function — nothing to push onto the stack
                    }
                    FunctionKind::Method { is_derived } => {
                        let instance = c.instance.as_option().ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;
                        self.define_local(Value::instance(instance))?;
                        if is_derived {
                            let class = class_val.as_option().ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;
                            self.define_local(Value::class(class))?;
                            if DEBUG_TRACE_EXECUTION {
                                eprintln!("Defining super!");
                            }
                        }
                    }
                }
            }
            _ if value.is_class() => {
                let class_val = value.unwrap_gc();
                let constructor = self.heap.resolve_class(class_val)
                    .map(|c| c.constructor)
                    .ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;

                if let Some(ctor_val) = constructor.as_option() {
                    let instance_val = self.heap.instance_create(class_val)
                        .ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;

                    self.define_local(instance_val)?;

                    let ctor = self.heap.resolve_closure(ctor_val).ok_or_else(|| EvaluateErrorDetails::ExpectedFunction)?;

                    let is_derived = if let FunctionKind::Method { is_derived } = ctor.function_kind {
                        is_derived
                    } else {
                        false
                    };

                    if is_derived {
                        // let _gc = class.base_class.ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;

                        self.define_local(Value::class(class_val))?;
                        if DEBUG_TRACE_EXECUTION {
                            eprintln!("Defining super!");
                        }
                    }
                }
            }
            _ if value.is_global_function() => {}
            _ => return Err(InterpretError::InvalidIdentifierType),
        }

        Ok(())
    }

    fn exec_set_base_class(&mut self) -> Result<(), InterpretError> {
        let value_register = self.read_register();
        let dist_register = self.read_register();

        let src_val = Some(self.core.registers[value_register as usize])
            .and_then(|c| c.is_class().then_some(c.unwrap_gc()))
            .ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;

        let dst_val = Some(self.core.registers[dist_register as usize])
            .and_then(|c| c.is_class().then_some(c.unwrap_gc()))
            .ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;


        self.heap.class_inherit(dst_val, src_val);

        Ok(())
    }

    fn exec_super(&mut self) -> Result<(), InterpretError> {
        let value_register = self.read_register();
        let this_register = self.read_register();
        let super_register = self.read_register();
        let dist_register = self.read_register();

        // `value_register` holds a string naming the method we want
        let identifier = self.heap.to_string(self.core.registers[value_register as usize]).ok_or_else(|| EvaluateErrorDetails::ExpectedString)?;
        // `super_register` holds the class value
        let class_gc = self.read_value(super_register as usize).and_then(|c|
            c
                .is_class()
                .then_some(c.unwrap_gc()))
                .ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;


        let base_class_gc = (self.heap.resolve_class(class_gc)).and_then(|c| c.base_class.as_option())
            .ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;
        let method_val = self
            .heap
            .class_get_method(base_class_gc, &identifier)
            .as_option()
            .ok_or_else(|| EvaluateErrorDetails::UndefinedVariable(identifier))?;

        let instance_val = self.read_value(this_register as usize).and_then(|i|
            i.is_instance().then_some(i.unwrap_gc()))
            .ok_or_else(|| EvaluateErrorDetails::ExpectedClassInstance)?;

        self.core.registers[dist_register as usize] = self.heap.bind_method(method_val, instance_val).ok_or_else(|| EvaluateErrorDetails::UnbindedMethod)?;

        Ok(())
    }
}

pub fn save_registers(registers: &mut Registers) -> Registers {
    std::mem::replace(registers, std::array::from_fn(|_| Value::null()))
}

impl Vm {
    pub fn new(chunk: &'static Chunk, globals_count: global_index_type) -> Self {
        Self {
            core: VmData {
                register_base: 0,
                ip: 0,
                registers: std::array::from_fn(|_| Value::null()),
                global_variables: vec![None; globals_count as usize],
                stack_states: ArrayVec::default(),
                stack: vec![Value::null(); STACK_MAX_SIZE as usize],
                stack_index: 0,
                call_stack: ArrayVec::default(),
                current_chunk: chunk,
                stack_base: 0,
            },
            heap: Heap::new()
        }
    }

    pub fn exec_return(&mut self) -> Result<(), InterpretError> {
        return Ok(())
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
        if DEBUG_TRACE_EXECUTION {
            let tmp = vm.core.ip;
            disassemble_instruction(&vm.heap, vm.core.current_chunk, vm.core.ip, previous_ip);
            previous_ip = tmp;
        }

        let instruction: Instructions = unsafe { std::mem::transmute(*vm.core.current_chunk.code.get_unchecked(vm.core.ip)) };
        vm.core.ip += 1;
        execute_instruction(vm, instruction).or_else(|err| Err(EvaluateError {
            error: err,
            line: vm.core.current_chunk.get_line(0 as usize) as line_type,
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
