use std::fmt::Display;

use crate::{
    Token,
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

    fn evaluate(&mut self) -> super::Result {
        self.ok(Some(match self.token.token {
            TokenKind::Number | TokenKind::String => match self.token.value {
                TokenValue::Number(v) => Value::Number(v),
                TokenValue::String(v) => Value::String(v.to_string()),
                _ => panic!("Got null token when evaluating literal"),
            },
            TokenKind::Keyword(Keyword::True) => Value::Boolean(true),
            TokenKind::Keyword(Keyword::False) => Value::Boolean(false),
            TokenKind::Keyword(Keyword::Nil) => Value::Null,
            _ => panic!("Invalid token considered as literal"),
        }))
    }
}
