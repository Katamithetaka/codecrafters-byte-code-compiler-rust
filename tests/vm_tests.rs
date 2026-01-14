
use interpreter::compiler::vm::{Value, Vm, interpret_with_vm};
use interpreter::compiler::instructions::Instructions;
use interpreter::compiler::chunk::Chunk;
use interpreter::expressions::Function;



#[test]
fn test_vm_negate_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index, 0);
    chunk.write_unary(Instructions::Negate, 0, 0, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.registers[0], Value::Number(-42.0));
}

#[test]
fn test_vm_add_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(10.0));
    let constant_index_1 = chunk.add_constant(Value::Number(32.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Add, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Number(42.0));
}

#[test]
fn test_vm_sub_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(50.0));
    let constant_index_1 = chunk.add_constant(Value::Number(8.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Sub, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Number(42.0));
}

#[test]
fn test_vm_mul_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(6.0));
    let constant_index_1 = chunk.add_constant(Value::Number(7.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Mul, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Number(42.0));
}

#[test]
fn test_vm_div_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(84.0));
    let constant_index_1 = chunk.add_constant(Value::Number(2.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Div, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Number(42.0));
}

#[test]
fn test_vm_return_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();

    // Define a function
    let function_start = chunk.code.len();
    let function_name = "test_function".to_string();
    let args_count = 0;



    // Add the function to the chunk's constants
    let function = Function::new(function_name, function_start as u16, args_count as u8);
    let function_constant = chunk.add_constant(Value::Function(function));

    // Write a jump to skip the function body
    let jump_offset = chunk.write_jump_placeholder(0);

    // Write function body
    let function_start = chunk.code.len();
    chunk.write_function_return(0); // Return from the function

    // Update the jump to skip the function body
    chunk.update_jump(jump_offset).unwrap();

    // Add the function to the chunk's constants
    let function_name = "test_function".to_string();
    let args_count = 0;
    let function = Function::new(function_name, function_start as u16, args_count as u8);
    let function_constant = chunk.add_constant(Value::Function(function));
    chunk.write_load(0, function_constant, 0);
    chunk.write_fn_call(0, 0, 0); // Call the function with 0 arguments
    chunk.disassemble("eval chunk");

    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(dbg!(result).is_ok());
}

#[test]
fn test_vm_return_outside_function() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    chunk.write_function_return(1); // Attempt to return outside of a function
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_err()); // Expect an error since return is not allowed outside a function
}

#[test]
fn test_vm_push_stack_instruction() {
    let mut vm = Box::new(Vm::new());
    vm.stack_index = 5;

    let mut chunk = Chunk::new();
    chunk.write_stack_push(0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.stack_states.len(), 1);
    assert_eq!(vm.stack_states[0], 5);
}

#[test]
fn test_vm_pop_stack_instruction() {
    let mut vm = Box::new(Vm::new());
    vm.stack_states.push(3);

    let mut chunk = Chunk::new();
    chunk.write_stack_pop(0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.stack_states.len(), 0);
    assert_eq!(vm.stack_index, 3);
}

#[test]
fn test_vm_define_global_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let key = chunk.add_constant(Value::String("test_var"));
    let value = Value::Number(42.0);

    vm.registers[0] = value.clone();
    chunk.write_constant(key, 123);

    chunk.write_declare_global(key, 0, 123);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.global_variables.get("test_var"), Some(&value));
}

#[test]
fn test_vm_bang_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index = chunk.add_constant(Value::Boolean(false));
    chunk.write_load(0, constant_index, 0);
    chunk.write_unary(Instructions::Bang, 0, 1, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[1], Value::Boolean(true));
}

#[test]
fn test_vm_lteq_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(42.0));
    let constant_index_1 = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::LtEq, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Boolean(true));
}

#[test]
fn test_vm_gteq_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(42.0));
    let constant_index_1 = chunk.add_constant(Value::Number(10.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::GtEq, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Boolean(true));
}

#[test]
fn test_vm_define_local_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index, 0);
    chunk.write_declare_local(0, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.stack[0], Value::Number(42.0));
}

#[test]
fn test_vm_get_local_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    vm.stack[0] = Value::Number(42.0);
    vm.stack_index += 1;
    
    chunk.write_get_local(0, 0, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(dbg!(result).is_ok());
    assert_eq!(vm.registers[0], Value::Number(42.0));
}

#[test]
fn test_vm_set_local_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index = chunk.add_constant(Value::Number(42.0));
    vm.stack[0] = Value::Number(69.0);
    vm.stack_index += 1;
    chunk.write_load(0, constant_index, 0);
    chunk.write_set_local(0, 0, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(dbg!(result).is_ok());
    assert_eq!(vm.stack[0], Value::Number(42.0));
}

#[test]
fn test_vm_print_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index, 0);
    chunk.write_print(0, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(dbg!(result).is_ok());
    // Note: To fully test this, you would need to capture stdout and verify the output.
}

#[test]
fn test_vm_debug_break_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    chunk.write_instruction(Instructions::DebugBreak, 0);
    // Note: This test assumes the debug break will wait for input. You may need to mock stdin to fully test this.
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
}

#[test]
fn test_vm_eq_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(42.0));
    let constant_index_1 = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Eq, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Boolean(true));
}

#[test]
fn test_vm_neq_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(42.0));
    let constant_index_1 = chunk.add_constant(Value::Number(10.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Neq, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Boolean(true));
}

#[test]
fn test_vm_lt_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(10.0));
    let constant_index_1 = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Lt, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Boolean(true));
}

#[test]
fn test_vm_gt_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_0 = chunk.add_constant(Value::Number(42.0));
    let constant_index_1 = chunk.add_constant(Value::Number(10.0));
    chunk.write_load(0, constant_index_0, 0);
    chunk.write_load(1, constant_index_1, 0);
    chunk.write_binary(Instructions::Gt, 0, 1, 2, 0);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[2], Value::Boolean(true));
}

#[test]
fn test_vm_jump_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    
    let constant_index = chunk.add_constant(Value::Number(69.0));
    chunk.write_load(0, constant_index, 0);
    let jump_offset = chunk.write_jump_placeholder(0);
    let constant_index = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index, 0);
    chunk.update_jump(jump_offset).unwrap();
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[0], Value::Number(69.0));
}

#[test]
fn test_vm_jump_if_false_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let constant_index_true = chunk.add_constant(Value::Boolean(false));
    let constant_index = chunk.add_constant(Value::Number(69.0));
    chunk.write_load(1, constant_index, 0);
    let constant_index_false = chunk.add_constant(Value::Number(42.0));
    chunk.write_load(0, constant_index_true, 0);
    let jump_offset = chunk.write_jump_if_false_placeholder(0, 0);
    chunk.write_load(1, constant_index_false, 0);
    chunk.update_jump(jump_offset).unwrap();
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());
    assert_eq!(vm.registers[1], Value::Number(69.0));
}

#[test]
fn test_vm_get_global_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let key = chunk.add_constant(Value::String("test_var"));
    let value = Value::Number(42.0);

    vm.global_variables.insert("test_var".to_string(), value.clone());
    chunk.write_get_global(key, 0, 123);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.registers[0], value);
}

#[test]
fn test_vm_set_global_instruction() {
    let mut vm = Box::new(Vm::new());
    let mut chunk = Chunk::new();
    let key = chunk.add_constant(Value::String("test_var"));
    let value = Value::Number(42.0);

    vm.global_variables.insert("test_var".to_string(), Value::Null);
    vm.global_variables.insert("test_var".to_string(), Value::Null);
    vm.registers[0] = value.clone();
    chunk.write_set_global(key, 0, 123);
    let result = interpret_with_vm(&mut vm, &chunk);
    assert!(result.is_ok());

    assert!(result.is_ok());
    assert_eq!(vm.global_variables.get("test_var"), Some(&value));
}

