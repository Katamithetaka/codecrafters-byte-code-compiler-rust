use std::fmt::Display;

use crate::{
    Token,
    compiler::{CodeGenerator, chunk::Chunk},
    expressions::{Expression, Value, expect_ok},
    scanner::{Keyword, TokenKind, TokenValue},
};

#[derive(Debug)]
pub struct Literal<'a> {
    token: &'a Token<'a>,
}

impl<'a> Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token.token {
            TokenKind::Number => write!(f, "{}", self.token.value),
            TokenKind::String => write!(f, "{}", self.token.value),
            TokenKind::Keyword(Keyword::True) => write!(f, "{}", self.token.lexeme),
            TokenKind::Keyword(Keyword::False) => write!(f, "{}", self.token.lexeme),
            TokenKind::Keyword(Keyword::Nil) => f.write_str("nil"),
            _ => f.write_str("GO FUCK YOURSELF"),
        }
    }
}

impl<'a> Literal<'a> {
    pub fn new(token: &'a Token<'a>) -> Self {
        return Self { token: token };
    }
}

impl<'a> Expression for Literal<'a> {
    fn line_number(&self) -> usize {
        self.token.line
    }
}

impl<'a> CodeGenerator for Literal<'a> {
    fn write_expression(
        &mut self,
        chunk: &mut Chunk,
        dst_register: Option<u8>,
        reserved_registers: Vec<u8>,
    ) -> crate::compiler::Result {
        let constant = match self.token.token {
            TokenKind::Number => match self.token.value {
                TokenValue::Number(v) => {
                    let constant =
                        chunk.get_or_write_constant(Value::Number(v), self.line_number() as i32);
                    constant
                }
                _ => panic!("Got null token when evaluating literal"),
            },
            TokenKind::String => match self.token.value {
                TokenValue::String(v) => {
                    let constant = chunk.get_or_write_constant(
                        Value::String(v.to_string()),
                        self.line_number() as i32,
                    );
                    constant
                }
                _ => panic!("Got null token when evaluating literal"),
            },
            TokenKind::Keyword(Keyword::True) => {
                let constant =
                    chunk.get_or_write_constant(Value::Boolean(true), self.line_number() as i32);
                constant
            }
            TokenKind::Keyword(Keyword::False) => {
                let constant =
                    chunk.get_or_write_constant(Value::Boolean(false), self.line_number() as i32);
                constant
            }
            TokenKind::Keyword(Keyword::Nil) => {
                let constant = chunk.get_or_write_constant(Value::Null, self.line_number() as i32);
                constant
            }
            _ => panic!("Invalid token considered as literal"),
        };

        match dst_register {
            Some(v) => {
                chunk.write_load(v, constant, self.line_number() as i32);
            }
            None => {}
        };

        Ok(())
    }
}
