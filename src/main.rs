#![allow(unused_variables)]
use interpreter::compiler::CodeGenerator;
use interpreter::compiler::compiler::Compiler;
use interpreter::compiler::instructions::Instructions;
use interpreter::compiler::vm::interpret;
use interpreter::global_functions::register_global_functions;
use interpreter::prelude::EvaluateError;
use interpreter::prelude::EvaluateErrorDetails;
use interpreter::*;
use std::env;
use std::fs;
use std::rc::Rc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        eprintln!("Usage: {} parse <filename>", args[0]);
        eprintln!("Usage: {} evaluate <filename>", args[0]);
        eprintln!("Usage: {} run <filename>", args[0]);

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

            let chunk = Compiler::new();

            match v.write_expression(chunk.clone(), Some(0), vec![]) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(70)
                }
            }

            let mut chunk = Rc::into_inner(chunk).unwrap().into_inner();
            chunk.write_print(0, 123);
            chunk.write_instruction(Instructions::Return, 123);
            chunk.disassemble("eval chunk");
            match interpret(Rc::new(chunk.chunk.into())) {
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


            let mut parser = AstParser::new(&tokens);
            let v = parser
                .parse();

            let v = match v {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(65)
                }
            };

            let chunk = Compiler::new();
            register_global_functions(&mut chunk.borrow_mut());
            for mut expr in v {
                match expr.write_expression(chunk.clone(), None, vec![]) {
                    Ok(_) => {}
                    Err(EvaluateError { error: EvaluateErrorDetails::ParserError(e), ..}) => {
                        eprintln!("Parser error: {e}");
                        std::process::exit(65)
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        std::process::exit(70)
                    }
                }
            }

            let mut chunk = Rc::into_inner(chunk).unwrap().into_inner();

            chunk.write_instruction(Instructions::Return, 123);
            chunk.disassemble("eval chunk");
            eprintln!("~eval chunk");
            match interpret(Rc::new(chunk.chunk.into())) {
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
