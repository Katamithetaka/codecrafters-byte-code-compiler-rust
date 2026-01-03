#![allow(unused_variables)]
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

            let eval = v.evaluate();

            let result = match eval {
                Ok(ok) => ok,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(70)
                }
            };

            println!("{}", result);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
