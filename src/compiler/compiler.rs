use std::{cell::RefCell, num::TryFromIntError, rc::Rc};

use crate::{ParserError, compiler::{instructions::{Instructions, disassemble_instruction}, varint::Varint}, expressions::Value, prelude::Chunk};

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
    pub index: u16,
}


#[allow(unused)]
pub struct Compiler<'a> {
    pub chunk: Chunk<&'a str>,

    pub locals: Vec<Local>,
    pub upvalues: Vec<UpvalueDesc>,

    pub scope_depth: i32,
    enclosing: Option<Rc<RefCell<Compiler<'a>>>>,
    globals: Option<Vec<String>>
}

pub enum ResolvedVar {
    Local(u16),
    Upvalue(u16),
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
            globals: Some(Vec::new())
        }))
    }

    pub fn with_parent(compiler: Rc<RefCell<Compiler<'a>>>) -> Rc<RefCell<Self>> {
        let compiler = Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            enclosing: Some(compiler),
            globals: None
        };



        Rc::new(RefCell::new(compiler))
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

    pub fn get_or_write_constant(&mut self, value: Value<&'a str>, line: i32) -> Varint {
        match self.get_constant(&value) {
            Some(constant) => constant,
            None => {
                let constant = self.add_constant(value);
                self.write_constant(constant, line as i32);
                constant
            }
        }
    }

    pub fn write(&mut self, byte: u8, line: i32) {
        self.chunk.write(byte, line);
    }

    pub fn write_instruction(&mut self, byte: Instructions, line: i32) {
        return self.write(byte as u8, line);
    }

    pub fn write_constant(&mut self, constant: Varint, line: i32) -> usize {
        self.write_instruction(Instructions::Constant, line);
        constant.write_bytes(self, line)
    }

    pub fn write_binary(&mut self, byte: Instructions, r0: u8, r1: u8, dst: u8, line: i32) {
        self.write(byte as u8, line);
        self.write(r0 as u8, line);
        self.write(r1 as u8, line);
        self.write(dst as u8, line);
    }

    pub fn write_load(&mut self, register_index: u8, constant: Varint, line: i32) -> usize {
        self.write_instruction(Instructions::Load, line);
        self.write(register_index as u8, line);
        constant.write_bytes(self, line)
    }

    pub fn write_jump_if_false_placeholder(&mut self, register_index: u8, line: i32) -> usize {
        self.write_instruction(Instructions::JumpIfFalse, line);
        self.write(register_index as u8, line);
        let return_val = self.chunk.code.len();
        self.write(0xFF as u8, line);
        self.write(0xFF as u8, line);
        return_val
    }

    pub fn write_jump_placeholder(&mut self, line: i32) -> usize {
        self.write_instruction(Instructions::Jump, line);
        let return_val = self.chunk.code.len();
        self.write(0xFF as u8, line);
        self.write(0xFF as u8, line);
        return_val
    }

    pub fn write_goto(&mut self, position: u16, line: i32) {
        self.write_instruction(Instructions::Jump, line);
        let values: [u8; 2] = position.to_be_bytes(); // IT IS NOW DECIDED THAT WE USE BIG ENDIAN LMAO
        self.write(values[0], line);
        self.write(values[1], line);
    }

    pub fn update_jump(&mut self, index: usize) -> Result<(), TryFromIntError> {
        let current_offset: u16 = self.chunk.code.len().try_into()?;
        let values: [u8; 2] = current_offset.to_be_bytes(); // IT IS NOW DECIDED THAT WE USE BIG ENDIAN LMAO
        self.chunk.code[index] = values[0];
        self.chunk.code[index + 1] = values[1];
        Ok(())
    }

    pub fn write_print(&mut self, register_index: u8, line: i32) {
        self.write_instruction(Instructions::Print, line);
        self.write(register_index as u8, line);
    }

    pub fn declare_variable(&mut self, name: &'a str, line: i32) -> Result<(), ()> {
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

    pub fn declare_function(&mut self, name: &'a str, line: i32)  {
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

    pub fn write_declare_global(&mut self, ident: Varint, value_register: u8, line: i32) -> usize {
        self.write_instruction(Instructions::DefineGlobal, line);
        self.write(value_register as u8, line);
        ident.write_bytes(self, line)
    }

    pub fn write_declare_local(&mut self, value_register: u8, line: i32) {
        let last = self.locals.last_mut().unwrap();
        last.depth = self.scope_depth;
        self.write_instruction(Instructions::DefineLocal, line);
        self.write(value_register as u8, line);
    }

    pub fn write_get_local(&mut self, output_register: u8, slot: u16, line: i32) {
        self.write_instruction(Instructions::GetLocal, line);
        self.write(output_register as u8, line);

        let slot: [u8; 2] = slot.to_be_bytes(); // IT IS NOW DECIDED THAT WE USE BIG ENDIAN LMAO

        self.write(slot[0] as u8, line);
        self.write(slot[1] as u8, line);

    }

    pub fn write_get_upvalue(&mut self, output_register: u8, slot: u16, line: i32) {
        self.write_instruction(Instructions::GetUpvalue, line);  // ← Fix this
        self.write(output_register as u8, line);
        let slot: [u8; 2] = slot.to_be_bytes();
        self.write(slot[0] as u8, line);
        self.write(slot[1] as u8, line);
    }

    pub fn write_set_upvalue(&mut self, input_register: u8, slot: u16, line: i32) {
        self.write_instruction(Instructions::SetUpvalue, line);  // ← Fix this
        self.write(input_register as u8, line);
        let slot: [u8; 2] = slot.to_be_bytes();
        self.write(slot[0] as u8, line);
        self.write(slot[1] as u8, line);
    }

    pub fn write_set_local(&mut self, input_register: u8, slot: u16, line: i32) {
        self.write_instruction(Instructions::SetLocal, line);
        self.write(input_register as u8, line);
        let slot: [u8; 2] = slot.to_be_bytes(); // IT IS NOW DECIDED THAT WE USE BIG ENDIAN LMAO

        self.write(slot[0] as u8, line);
        self.write(slot[1] as u8, line);
    }



    pub fn write_set_global(&mut self, ident: Varint, value_register: u8, line: i32) {
        self.write_instruction(Instructions::SetGlobal, line);
        self.write(value_register as u8, line);
        ident.write_bytes(self, line);
    }

    pub fn write_get_global(&mut self, ident: Varint, dst_register: u8, line: i32) -> usize {
        self.write_instruction(Instructions::GetGlobal, line);
        self.write(dst_register as u8, line);
        ident.write_bytes(self, line)
    }

    pub fn write_function_return(&mut self, line: i32) {
        self.write_instruction(Instructions::FunctionReturn, line);
    }

    pub fn write_stack_push(&mut self, line: i32) {
        self.scope_depth += 1;
        self.write_instruction(Instructions::PushStack, line);
    }

    pub fn write_stack_pop(&mut self, line: i32) {
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



    pub fn write_fn_call(&mut self, fn_register: u8, num_args: u8, line: i32) {
        self.write_instruction(Instructions::FunctionCall, line);
        self.write(fn_register, line);
        self.write(num_args, line);

    }


    pub fn write_unary(
        &mut self,
        byte: Instructions,
        register_index: u8,
        dst_register_index: u8,
        line: i32,
    ) {
        self.write(byte as u8, line);
        self.write(register_index as u8, line);
        self.write(dst_register_index as u8, line);
    }

    pub fn disassemble(&self, name: &str) {
        eprintln!("== {} ==", name);
        let mut i = 0;
        let mut previous = i;
        while i < self.chunk.code.len() {
            let tmp = i;
            i = disassemble_instruction(&self.chunk, i, previous);
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

    fn resolve_full_upvalue(&mut self, name: &str) -> Option<u16> {
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

    fn resolve_predeclared_upvalue(&mut self, name: &str) -> Option<u16> {
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

    fn resolve_local(&mut self, name: &str) -> Result<Option<(u16, Local)>, ParserError> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name && local.depth != -1 {
                return Ok(Some((i as u16, local.clone())));
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

    pub fn add_upvalue(&mut self, is_local: bool, index: u16) -> u16 {
        // check if this upvalue already exists
        for (i, up) in self.upvalues.iter().enumerate() {
            if up.is_local == is_local && up.index == index {
                return i as u16; // reuse existing upvalue
            }
        }

        // otherwise, add new upvalue
        self.upvalues.push(UpvalueDesc { is_local, index });
        (self.upvalues.len() - 1) as u16
    }

    pub fn write_closure(&mut self, dst_reg: u8, constant: Varint, upvalues: &[UpvalueDesc], line: i32) {
        self.write_instruction(Instructions::Closure, line);
        self.write(dst_reg, line);

        // Write the constant index as varint
        constant.write_bytes(self, line);

        // Write the number of upvalues
        self.write(upvalues.len() as u8, line);

        // Write each upvalue's metadata
        for upvalue in upvalues {
            // Write whether it's a local (1) or inherited from parent (0)
            self.write(if upvalue.is_local { 1 } else { 0 }, line);

            // Write the index as u16 (big endian to match your other code)
            let index_bytes = upvalue.index.to_be_bytes();
            self.write(index_bytes[0], line);
            self.write(index_bytes[1], line);
        }
    }
}
