use std::{borrow::Cow, fmt::Display, iter::Peekable, str::Chars};

use strum::{Display, IntoStaticStr};

#[derive(Display, IntoStaticStr, Debug, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenKind {
    LeftParen,
    RightParen,
    Number,
    String,
    #[strum(serialize = "EOF")]
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenValue<'a> {
    Null,
    Number(f64),
    String(&'a str),
}

impl<'a> Display for TokenValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Null => write!(f, "null"),
            TokenValue::Number(v) => write!(f, "{:.1}", v),
            TokenValue::String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
pub struct Token<'a> {
    token: TokenKind,
    lexeme: &'a str,
    value: TokenValue<'a>,
    line: usize,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.token, self.lexeme, self.value)
    }
}

struct Lexer<'a> {
    input: &'a str,
    it: Peekable<Chars<'a>>,
    line: usize,
    pos: usize,
    end: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(v: &'a str) -> Self {
        Self {
            input: v,
            it: v.chars().peekable(),
            line: 1,
            pos: 0,
            end: false,
        }
    }

    pub fn advance(&mut self) -> Option<char> {
        self.pos += 1;
        return self.it.next();
    }

    pub fn consume_new_line(&mut self) {
        self.advance().unwrap();
        self.line += 1;
    }

    pub fn consume_single_char(&mut self, token: TokenKind, v: &'a str) -> Token<'a> {
        self.advance().unwrap();
        return Token {
            token,
            line: self.line,
            lexeme: v,
            value: TokenValue::Null,
        };
    }

    pub fn consume_number(&mut self) -> Token<'a> {
        let begin_pos = self.pos;
        while let Some(v) = self.it.peek() {
            if v.is_ascii_digit() || *v == '.' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.input[begin_pos..self.pos];
        let value = TokenValue::Number(lexeme.parse().unwrap());
        return Token {
            token: TokenKind::Number,
            lexeme,
            value,
            line: self.line,
        };
    }

    pub fn consume_string(&mut self) -> Token<'a> {
        let begin_pos = self.pos;
        let mut found_end = false;
        self.advance();
        while let Some(v) = self.it.peek() {
            if *v == '"' {
                found_end = true;
                break;
            } else {
                self.advance();
            }
        }

        if !found_end {
            panic!("Never found string end!");
        }

        let lexeme = &self.input[begin_pos..self.pos];
        let value = TokenValue::String(&lexeme[1..=lexeme.len() - 1]); // remove quotes
        return Token {
            token: TokenKind::String,
            lexeme,
            value,
            line: self.line,
        };
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.end {
                return None;
            }
            match self.it.peek() {
                Some('(') => return self.consume_single_char(TokenKind::LeftParen, "(").into(),
                Some(')') => return self.consume_single_char(TokenKind::RightParen, ")").into(),
                Some('\n') => self.consume_new_line(),
                Some(_) => panic!("Unexpected token"),
                None => {
                    self.end = true;
                    return Some(Token {
                        token: TokenKind::EOF,
                        lexeme: "",
                        value: TokenValue::Null,
                        line: self.line,
                    });
                }
            }
        }
    }
}

pub fn tokenize(value: &str) -> Vec<Token<'_>> {
    let lexer = Lexer::new(value);

    return lexer.into_iter().collect();
}

pub mod prelude {
    pub use super::Token;
    pub use super::tokenize;
}
