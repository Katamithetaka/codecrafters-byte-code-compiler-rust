#![allow(unused_variables)]
use interpreter::compiler::CodeGenerator;
use interpreter::compiler::chunk::Chunk;
use interpreter::compiler::instructions::Instructions;
use interpreter::compiler::vm::interpret;
use interpreter::global_functions::register_global_functions;
use interpreter::resolver::Resolver;
use interpreter::*;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            // TODO: Uncomment the code below to pass the first stage
            let tokenizing_error = {
                let mut tokenizing_error = false;
                for token in tokenize(&file_contents) {
                    match token.as_ref() {
                        Ok(v) => {
                            println!("{}", v);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            tokenizing_error = true;
                        }
                    }
                }
                tokenizing_error
            };

            if tokenizing_error {
                std::process::exit(65);
            }
        }
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let tokens = {
                tokenize(&file_contents)
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
            };

            let tokens = match tokens {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            let mut parser = AstParser::new(&tokens);
            let v = parser.expression();

            let v = match v {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            println!("{v}");
        }
        "evaluate" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let tokens = {
                tokenize(&file_contents)
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
            };

            let tokens = match tokens {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            let mut parser = AstParser::new(&tokens);
            let v = parser.expression();

            let mut v = match v {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            let mut chunk = Chunk::new();

            match v.write_expression(&mut chunk, Some(0), vec![]) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(70)
                }
            }
            chunk.write_print(0, 123);
            chunk.write_instruction(Instructions::Return, 123);
            chunk.disassemble("eval chunk");
            match interpret(&chunk) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(70)
                }
            }
        }
        "run" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let tokens = {
                tokenize(&file_contents)
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
            };

            let tokens = match tokens {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            let mut resolver = Resolver::new();

            let mut parser = AstParser::new(&tokens);
            let v = parser
                .parse()
                .map(|tokens| resolver.resolve_statements(tokens))
                .flatten();

            let v = match v {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            let mut chunk = Chunk::new();
            register_global_functions(&mut chunk);
            for mut expr in v {
                match expr.write_expression(&mut chunk, None, vec![]) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("{err}");
                        std::process::exit(70)
                    }
                }
            }
            chunk.write_instruction(Instructions::Return, 123);
            chunk.disassemble("eval chunk");
            eprintln!("~eval chunk");
            match interpret(&chunk) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(70)
                }
            }
        }

        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

