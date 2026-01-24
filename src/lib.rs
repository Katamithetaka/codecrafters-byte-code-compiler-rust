#![doc = r##"
# Codecrafters Interpreter in Rust

This library is an implementation of a Lox interpreter written in Rust. Lox is a dynamically-typed programming language introduced in the book *Crafting Interpreters* by Robert Nystrom. This interpreter supports parsing, resolving, and executing Lox scripts.

## Features

- **Lexer**: Tokenizes the input source code into a stream of tokens.
- **Parser**: Converts the token stream into an Abstract Syntax Tree (AST).
- **Resolver**: Handles variable scoping and resolves identifiers.
- **Interpreter**: Executes the parsed Lox code.

## Example Usage

The following example demonstrates how to interpret a Lox script that calculates the Fibonacci sequence:

```rust
use interpreter::prelude::*;
use interpreter::compiler::chunk::Chunk;
use interpreter::global_functions::register_global_functions;
use interpreter::compiler::vm::interpret;
fn main() {
    let source = r#"
    fun fib(n) {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    }

    print fib(10); // Outputs: 55
    "#;

    // Step 1: Tokenize the source code
    let tokens = {
        tokenize(&source)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }.expect("Failed to parse tokens");

    // Step 2: Parse the tokens into an AST
    let mut parser = AstParser::new(&tokens);
    let statements = parser.parse().expect("Failed to parse");


    // Step 3: Compile the resolved AST into bytecode
    let mut chunk = Chunk::new();
    register_global_functions(&mut chunk);
    for mut statement in resolved_statements {
        statement.write_expression(&mut chunk, None, vec![]).expect("Failed to compile");
    }

    // Step 4: Interpret the bytecode
    let result = interpret(&chunk).expect("Failed to interpret");
    println!("Result: {:?}", result);
}
```

## Modules

- `ast_parser`: Handles parsing of tokens into an Abstract Syntax Tree (AST).
- `compiler`: Contains the bytecode compiler and related utilities.
- `expressions`: Defines the various expressions in the Lox language.
- `resolver`: Manages variable scoping and identifier resolution.
- `scanner`: Tokenizes the source code into a stream of tokens.
- `statements`: Defines the various statements in the Lox language.
- `global_functions`: Provides built-in global functions for the interpreter.

## Getting Started

To use this library, add it to your `Cargo.toml`:

```toml
[dependencies]
codecrafters-interpreter-rust = "0.1.0"
```

Then, import the necessary modules and start interpreting Lox scripts!

For more details, refer to the documentation of individual modules and types.
"##]

mod ast_parser;
pub mod compiler;
pub mod expressions;

mod scanner;
pub mod statements;
pub mod global_functions;
pub mod value;

pub use ast_parser::prelude::*;
pub use scanner::prelude::*;

pub mod prelude {
    pub use crate::ast_parser::prelude::*;
    pub use crate::scanner::prelude::*;
    pub use crate::compiler::chunk::Chunk;
    pub use crate::compiler::CodeGenerator;
    pub use crate::expressions::prelude::*;
    pub use crate::statements::prelude::*;
    pub use crate::global_functions::prelude::*;
}
