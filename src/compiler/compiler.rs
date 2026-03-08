use std::{cell::RefCell,  rc::Rc};

use crate::{ParserError, compiler::{instructions::{Instructions, disassemble_instruction}, int_types::{instruction_length_type, line_type, register_index_type, stack_index_type}, varint::Varint}, expressions::Value, prelude::{Chunk, EvaluateError}, value::callable::FunctionKind};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: i32,
    pub is_captured: bool,
    pub is_predeclared: bool
}

#[allow(unused)]
pub struct UpvalueDesc {
    pub is_local: bool,
    pub index: stack_index_type,
}


#[allow(unused)]
pub struct Compiler<'a> {
    pub chunk: Chunk<&'a str>,

    pub locals: Vec<Local>,
    pub upvalues: Vec<UpvalueDesc>,

    pub scope_depth: i32,
    pub enclosing: Option<Rc<RefCell<Compiler<'a>>>>,
    pub function_kind: Option<FunctionKind>,
    pub function_name: Option<String>,
    globals: Option<Vec<String>>
}

pub enum ResolvedVar {
    Local(stack_index_type),
    Upvalue(stack_index_type),
    Global(Varint),
}

impl<'a> Compiler<'a> {
    pub fn new() -> Rc<RefCell<Self>>  {
        Rc::new(RefCell::new(Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            enclosing: None,
            globals: Some(Vec::new()),
            function_kind: None,
            function_name: None
        }))
    }

    pub fn with_parent(compiler: Rc<RefCell<Compiler<'a>>>, function_name: String, function_kind: FunctionKind) -> Rc<RefCell<Self>> {
        let compiler = Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            enclosing: Some(compiler),
            globals: None,
            function_kind: Some(function_kind),
            function_name: Some(function_name)
        };



        Rc::new(RefCell::new(compiler))
    }

    pub fn is_in_method(&self) -> bool {
        match self.function_kind {
            Some(c) => c == FunctionKind::Method || self.enclosing.as_ref().unwrap().borrow().is_in_method(),
            None => false,
        }
    }

    pub fn is_in_constructor(&self) -> bool {
        match (self.function_kind, &self.function_name) {
            (Some(c), Some(function_name)) => c == FunctionKind::Method && function_name == "init",
            _ => false,
        }
    }

    fn add_global(&mut self, name: String) {
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().add_global(name),
            None => self.globals.as_mut().unwrap().push(name),
        }
    }

    fn has_global(&self, name: &String) -> bool {
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow().has_global(name),
            None => self.globals.as_ref().unwrap().contains(&name),
        }
    }

    pub fn get_constant(&mut self, value: &Value<&'a str>) -> Option<Varint> {
        return self.chunk.get_constant(value)
    }

    pub fn add_constant(&mut self, value: Value<&'a str>) -> Varint {
        return self.chunk.add_constant(value)
    }

    pub fn get_or_write_constant(&mut self, value: Value<&'a str>, line: line_type) -> Varint {
        match self.get_constant(&value) {
            Some(constant) => constant,
            None => {
                let constant = self.add_constant(value);
                self.write_constant(constant, line);
                constant
            }
        }
    }

    pub fn write(&mut self, byte: u8, line: line_type) {
        self.chunk.write(byte, line);
    }

    pub fn write_bytes(&mut self, bytes: &[u8], line: line_type) {
        for byte in bytes {
            self.chunk.write(*byte, line);
        }
    }



    pub fn write_instruction(&mut self, byte: Instructions, line: line_type) {
        return self.write(byte as u8, line);
    }

    pub fn write_constant(&mut self, constant: Varint, line: line_type) -> usize {
        self.write_instruction(Instructions::Constant, line);
        constant.write_bytes(self, line)
    }

    pub fn write_binary(&mut self, byte: Instructions, r0: register_index_type, r1: register_index_type, dst: register_index_type, line: line_type) {
        self.write(byte as u8, line);
        self.write_bytes(&r0.to_be_bytes(), line);
        self.write_bytes(&r1.to_be_bytes(), line);
        self.write_bytes(&dst.to_be_bytes(), line);
    }

    pub fn write_load(&mut self, register_index: register_index_type, constant: Varint, line: line_type) -> usize {
        self.write_instruction(Instructions::Load, line);
        self.write_bytes(&register_index.to_be_bytes(), line);
        constant.write_bytes(self, line)
    }

    pub fn write_jump_if_false_placeholder(&mut self, register_index: register_index_type, line: line_type) -> Result<usize, EvaluateError> {
        self.write_instruction(Instructions::JumpIfFalse, line);
        self.write_bytes(&register_index.to_be_bytes(), line);
        let return_val = self.chunk.code.len();
        let current_offset: instruction_length_type = self.chunk.code.len().try_into().map_err(|_| EvaluateError {
            error: crate::value::EvaluateErrorDetails::CodeTooLong,
            line: line,
        })?;

        let values = current_offset.to_be_bytes();

        for _ in 0..values.len() {
            self.write(0xFF as u8, line);

        }
        Ok(return_val)

    }

    pub fn write_jump_placeholder(&mut self, line: line_type) -> Result<usize, EvaluateError> {
        self.write_instruction(Instructions::Jump, line);
        let return_val = self.chunk.code.len();
        let current_offset: instruction_length_type = self.chunk.code.len().try_into().map_err(|_| EvaluateError {
            error: crate::value::EvaluateErrorDetails::CodeTooLong,
            line: line,
        })?;
        let values = current_offset.to_be_bytes();

        for _ in 0..values.len() {
            self.write(0xFF as u8, line);
        }

        Ok(return_val)
    }

    pub fn write_goto(&mut self, position: instruction_length_type, line: line_type) {
        self.write_instruction(Instructions::Jump, line);
        let values = position.to_be_bytes();
        self.write_bytes(&values, line);
    }

    pub fn update_jump(&mut self, index: usize) -> Result<(), EvaluateError> {
        let current_offset: instruction_length_type = self.chunk.code.len().try_into().map_err(|_| EvaluateError {
            error: crate::value::EvaluateErrorDetails::CodeTooLong,
            line: 0,
        })?;
        let values = current_offset.to_be_bytes();
        for i in 0..values.len() {
            self.chunk.code[index + i] = values[i];
        }
        Ok(())
    }

    pub fn write_print(&mut self, register_index: register_index_type, line: line_type) {
        self.write_instruction(Instructions::Print, line);
        self.write_bytes(&register_index.to_be_bytes(), line);
    }

    pub fn declare_variable(&mut self, name: &'a str, line: line_type) -> Result<(), ()> {
        if self.scope_depth == 0 && !self.enclosing.is_some() {
            self
                .get_or_write_constant(Value::String(name), line);
            self.add_global(name.to_string());
        } else {
            if self.locals.iter().any(|f| f.name == name.to_string() && f.depth == self.scope_depth) {
                return Err(());
            }
            self.locals.push(Local { name: name.to_string(), depth: -1, is_captured: false, is_predeclared: false, });
        }

        Ok(())
    }

    pub fn declare_function(&mut self, name: &'a str, line: line_type)  {
        if self.scope_depth == 0 && !self.enclosing.is_some() {
            self
                .get_or_write_constant(Value::String(name), line);
            self.add_global(name.to_string());
        } else {
            if self.locals.iter().any(|f| f.name == name.to_string()) {
                return;
            }
            self.locals.push(Local { name: name.to_string(), depth: self.scope_depth, is_captured: false, is_predeclared: true, });
        }
    }

    pub fn write_declare_global(&mut self, ident: Varint, value_register: register_index_type, line: line_type) -> usize {
        self.write_instruction(Instructions::DefineGlobal, line);
        self.write_bytes(&value_register.to_be_bytes(), line);
        ident.write_bytes(self, line)
    }

    pub fn write_declare_local(&mut self, value_register: register_index_type, line: line_type) {
        let last = self.locals.last_mut().unwrap();
        last.depth = self.scope_depth;
        self.write_instruction(Instructions::DefineLocal, line);
        self.write_bytes(&value_register.to_be_bytes(), line);
    }

    pub fn write_get_local(&mut self, output_register: register_index_type, slot: stack_index_type, line: line_type) {
        self.write_instruction(Instructions::GetLocal, line);
        self.write_bytes(&output_register.to_be_bytes(), line);
        self.write_bytes(&slot.to_be_bytes(), line);

    }

    pub fn write_get_upvalue(&mut self, output_register: register_index_type, slot: stack_index_type, line: line_type) {
        self.write_instruction(Instructions::GetUpvalue, line);  // ← Fix this
        self.write_bytes(&output_register.to_be_bytes(), line);
        self.write_bytes(&slot.to_be_bytes(), line);

    }

    pub fn write_set_upvalue(&mut self, input_register: register_index_type, slot: stack_index_type, line: line_type) {
        self.write_instruction(Instructions::SetUpvalue, line);  // ← Fix this
        self.write_bytes(&input_register.to_be_bytes(), line);
        self.write_bytes(&slot.to_be_bytes(), line);

    }

    pub fn write_set_local(&mut self, input_register: register_index_type, slot: stack_index_type, line: line_type) {
        self.write_instruction(Instructions::SetLocal, line);
        self.write_bytes(&input_register.to_be_bytes(), line);
        self.write_bytes(&slot.to_be_bytes(), line);

    }



    pub fn write_set_global(&mut self, ident: Varint, value_register: register_index_type, line: line_type) {
        self.write_instruction(Instructions::SetGlobal, line);
        self.write_bytes(&value_register.to_be_bytes(), line);
        ident.write_bytes(self, line);
    }

    pub fn write_get_global(&mut self, ident: Varint, dst_register: register_index_type, line: line_type) -> usize {
        self.write_instruction(Instructions::GetGlobal, line);
        self.write_bytes(&dst_register.to_be_bytes(), line);
        ident.write_bytes(self, line)
    }

    pub fn write_get_field(&mut self, ident: Varint, dst_register: register_index_type, line: line_type) -> usize {
        self.write_instruction(Instructions::GetField, line);
        self.write_bytes(&dst_register.to_be_bytes(), line);
        ident.write_bytes(self, line)
    }

    pub fn write_set_field(&mut self, ident: Varint, value_register: register_index_type, dst_register: register_index_type, line: line_type) -> usize {
        self.write_instruction(Instructions::SetField, line);
        self.write_bytes(&value_register.to_be_bytes(), line);
        self.write_bytes(&dst_register.to_be_bytes(), line);
        ident.write_bytes(self, line)
    }

    pub fn write_method_declare(&mut self, func_register: register_index_type, dst_register: register_index_type, line: line_type) {
        self.write_instruction(Instructions::CreateMethod, line);
        self.write_bytes(&func_register.to_be_bytes(), line);
        self.write_bytes(&dst_register.to_be_bytes(), line);
    }

    pub fn write_function_init(&mut self, func_register: register_index_type, line: line_type) {
        self.write_instruction(Instructions::InitFunction, line);
        self.write_bytes(&func_register.to_be_bytes(), line);

    }

    pub fn write_function_return(&mut self, line: line_type) {
        self.write_instruction(Instructions::FunctionReturn, line);
    }

    pub fn write_stack_push(&mut self, line: line_type) {
        self.scope_depth += 1;
        self.write_instruction(Instructions::PushStack, line);
    }

    pub fn write_stack_pop(&mut self, line: line_type) {
        while let Some(local) = self.locals.last() {
            if local.depth < self.scope_depth {
                break;
            }

            if local.is_captured {
                // self.emit_close_upvalue();
            } else {
                // self.emit_pop();
            }

            self.locals.pop();
        }

        self.scope_depth -= 1;

        self.write_instruction(Instructions::PopStack, line);
    }



    pub fn write_fn_call(&mut self, fn_register: register_index_type, num_args: u8, line: line_type) {
        self.write_instruction(Instructions::FunctionCall, line);
        self.write_bytes(&fn_register.to_be_bytes(), line);
        self.write(num_args, line);

    }


    pub fn write_unary(
        &mut self,
        byte: Instructions,
        register_index: register_index_type,
        dst_register_index: register_index_type,
        line: line_type,
    ) {
        self.write(byte as u8, line);
        self.write_bytes(&register_index.to_be_bytes(), line);
        self.write_bytes(&dst_register_index.to_be_bytes(), line);

    }

    pub fn disassemble(&self, name: &str) {
        eprintln!("== {} ==", name);
        let mut i = 0;
        let mut previous = i;
        while i < self.chunk.code.len() {
            let tmp = i;
            i = disassemble_instruction(Rc::new(self.chunk.clone()), i, previous);
            previous = tmp;
        }
    }

    pub fn mark_declared(&mut self, name: String) {
        match self.locals.iter_mut().find(|l| l.name == name) {
            Some(v) => v.is_predeclared = false,
            None => {},
        }
    }

    pub fn resolve_variable(&mut self, name: &'a str) -> Result<ResolvedVar, ParserError> {
        if let Some((slot, _)) = self.resolve_local(name)? {
            return Ok(ResolvedVar::Local(slot));
        }

        if let Some(up) = self.resolve_full_upvalue(name) {
            return Ok(ResolvedVar::Upvalue(up));
        }

        if self.has_global(&name.to_string()) {
            let global = self.get_or_write_constant(
                Value::String(name),
                0,
            );

            return Ok(ResolvedVar::Global(global))
        }

        if let Some(up) = self.resolve_predeclared_upvalue(name) {
            return Ok(ResolvedVar::Upvalue(up));
        }

        let global = self.get_or_write_constant(
            Value::String(name),
            0,
        );

        return Ok(ResolvedVar::Global(global))
    }

    fn resolve_full_upvalue(&mut self, name: &str) -> Option<stack_index_type> {
        let mut enclosing = self.enclosing.as_mut()?.borrow_mut();

        if let Some((slot, local)) = enclosing.resolve_local(name).unwrap() {
            if local.is_predeclared {
                if let Some(up) = enclosing.resolve_full_upvalue(name) {
                    drop(enclosing);
                    return Some(self.add_upvalue(false, up));
                }
                else {
                    return None;
                }
            }
            enclosing.locals[slot as usize].is_captured = true;
            drop(enclosing);
            return Some(self.add_upvalue(true, slot));
        }

        if let Some(up) = enclosing.resolve_full_upvalue(name) {
            drop(enclosing);
            return Some(self.add_upvalue(false, up));
        }

        None
    }

    fn resolve_predeclared_upvalue(&mut self, name: &str) -> Option<stack_index_type> {
        let mut enclosing = self.enclosing.as_mut()?.borrow_mut();

        if let Some((slot, _)) = enclosing.resolve_local(name).unwrap() {

            enclosing.locals[slot as usize].is_captured = true;
            drop(enclosing);
            return Some(self.add_upvalue(true, slot));
        }

        if let Some(up) = enclosing.resolve_full_upvalue(name) {
            drop(enclosing);
            return Some(self.add_upvalue(false, up));
        }

        None
    }

    fn resolve_local(&mut self, name: &str) -> Result<Option<(stack_index_type, Local)>, ParserError> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name && local.depth != -1 {
                return Ok(Some((i as stack_index_type, local.clone())));
            }
            if local.name == name && local.depth == -1 {
                return Err(ParserError {
                    error: crate::ast_parser::ParserErrorDetails::InvalidAssignementTarget,
                    line: 0
                });
            }
        }

        Ok(None)
    }

    pub fn add_upvalue(&mut self, is_local: bool, index: stack_index_type) -> stack_index_type {
        // check if this upvalue already exists
        for (i, up) in self.upvalues.iter().enumerate() {
            if up.is_local == is_local && up.index == index {
                return i as stack_index_type; // reuse existing upvalue
            }
        }

        // otherwise, add new upvalue
        self.upvalues.push(UpvalueDesc { is_local, index });
        (self.upvalues.len() - 1) as stack_index_type
    }

    pub fn write_closure(&mut self, dst_reg: register_index_type, constant: Varint, upvalues: &[UpvalueDesc], line: line_type) {
        self.write_instruction(Instructions::Closure, line);
        self.write_bytes(&dst_reg.to_be_bytes(), line);

        // Write the constant index as varint
        constant.write_bytes(self, line);

        // Write the number of upvalues
        self.write(upvalues.len() as u8, line);

        // Write each upvalue's metadata
        for upvalue in upvalues {
            // Write whether it's a local (1) or inherited from parent (0)
            self.write(if upvalue.is_local { 1 } else { 0 }, line);

            let index_bytes = upvalue.index.to_be_bytes();
            self.write_bytes(&index_bytes, line);
        }
    }
}
