use std::{fmt::Display, iter::Peekable, str::Chars};

use strum::{Display, IntoStaticStr};

#[derive(thiserror::Error, Debug)]
pub enum ScanningError {
    #[error("[line {0}] Error: Unterminated string.")]
    UnterminatedString(usize),
    #[error("[line {0}] Error: Parsing number literal failed")]
    InvalidNumber(usize),
    #[error("[line {0}] Error: Unexpected character: {1}")]
    LexicalError(usize, char),
}

#[derive(Display, IntoStaticStr, Debug, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Number,
    String,
    Star,
    Dot,
    Plus,
    Comma,
    Minus,
    Semicolon,
    Slash,
    Equal,
    Less,
    Greater,
    Bang,
    LessEqual,
    GreaterEqual,
    BangEqual,
    EqualEqual,
    Identifier,
    #[strum(serialize = "EOF")]
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenValue<'a> {
    Null,
    Number(f64),
    String(&'a str),
}

fn is_alpha(c: char) -> bool {
    return c.is_ascii_alphabetic() || c == '_';
}

impl<'a> Display for TokenValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Null => write!(f, "null"),
            TokenValue::Number(v) => {
                if v.fract() == 0.0 {
                    write!(f, "{:.1}", v)
                } else {
                    write!(f, "{}", v)
                }
            }
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

    pub fn consume_two(
        &mut self,
        two_char_token: (&'a str, TokenKind),
        single_char_token: (&'a str, TokenKind),
        peek_char: char,
    ) -> Token<'a> {
        self.advance().unwrap();
        if let Some(c) = self.it.peek() {
            if *c == peek_char {
                self.advance().unwrap();
                return Token {
                    token: two_char_token.1,
                    lexeme: two_char_token.0,
                    value: TokenValue::Null,
                    line: self.line,
                };
            } else {
                return Token {
                    token: single_char_token.1,
                    lexeme: single_char_token.0,
                    value: TokenValue::Null,
                    line: self.line,
                };
            }
        } else {
            return Token {
                token: single_char_token.1,
                lexeme: single_char_token.0,
                value: TokenValue::Null,
                line: self.line,
            };
        }
    }

    pub fn consume_number(&mut self) -> Result<Token<'a>, ScanningError> {
        let begin_pos = self.pos;
        while let Some(v) = self.it.peek() {
            if v.is_ascii_digit() || *v == '.' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.input[begin_pos..self.pos];
        let value = TokenValue::Number(
            lexeme
                .parse()
                .map_err(|_| ScanningError::InvalidNumber(self.line))?,
        );
        return Ok(Token {
            token: TokenKind::Number,
            lexeme,
            value,
            line: self.line,
        });
    }

    pub fn consume_string(&mut self) -> Result<Token<'a>, ScanningError> {
        let begin_pos = self.pos;
        let begin_line = self.line;
        let mut found_end = false;
        self.advance();
        while let Some(v) = self.it.peek() {
            if *v == '"' {
                found_end = true;
                self.advance();
                break;
            } else {
                if *v == '\n' {
                    self.consume_new_line();
                } else {
                    self.advance();
                }
            }
        }

        if !found_end {
            return Err(ScanningError::UnterminatedString(begin_line));
        }

        let lexeme = &self.input[begin_pos..self.pos];
        let value = TokenValue::String(&lexeme[1..=lexeme.len() - 2]); // remove quotes
        return Ok(Token {
            token: TokenKind::String,
            lexeme,
            value,
            line: self.line,
        });
    }

    pub fn consume_identifier(&mut self) -> Result<Token<'a>, ScanningError> {
        let begin_pos = self.pos;
        let mut found_end = false;
        self.advance();
        while let Some(v) = self.it.peek() {
            if !is_alpha(*v) && !v.is_ascii_digit() {
                break;
            } else {
                self.advance();
            }
        }

        let lexeme = &self.input[begin_pos..self.pos];
        return Ok(Token {
            token: TokenKind::Identifier,
            lexeme,
            value: TokenValue::Null,
            line: self.line,
        });
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, ScanningError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.end {
                return None;
            }
            match self.it.peek() {
                Some('(') => return Ok(self.consume_single_char(TokenKind::LeftParen, "(")).into(),
                Some(')') => {
                    return Ok(self.consume_single_char(TokenKind::RightParen, ")")).into();
                }

                Some('+') => return Ok(self.consume_single_char(TokenKind::Plus, "+")).into(),
                Some('-') => return Ok(self.consume_single_char(TokenKind::Minus, "-")).into(),
                Some('*') => return Ok(self.consume_single_char(TokenKind::Star, "*")).into(),
                Some('/') => {
                    let _ = self.advance().unwrap();
                    match self.it.peek() {
                        Some('/') => loop {
                            if let Some(next) = self.it.peek() {
                                if *next == '\n' {
                                    self.consume_new_line();
                                    break;
                                } else {
                                    self.advance();
                                }
                            } else {
                                self.end = true;
                                return Some(Ok(Token {
                                    token: TokenKind::EOF,
                                    lexeme: "",
                                    value: TokenValue::Null,
                                    line: self.line,
                                }));
                            }
                        },
                        _ => {
                            return Ok(Token {
                                token: TokenKind::Slash,
                                lexeme: "/",
                                value: TokenValue::Null,
                                line: self.line,
                            })
                            .into();
                        }
                    }
                }
                Some(',') => return Ok(self.consume_single_char(TokenKind::Comma, ",")).into(),
                Some(';') => return Ok(self.consume_single_char(TokenKind::Semicolon, ";")).into(),
                Some('.') => return Ok(self.consume_single_char(TokenKind::Dot, ".")).into(),

                Some('{') => return Ok(self.consume_single_char(TokenKind::LeftBrace, "{")).into(),
                Some('}') => {
                    return Ok(self.consume_single_char(TokenKind::RightBrace, "}")).into();
                }
                Some('>') => {
                    return Ok(self.consume_two(
                        (">=", TokenKind::GreaterEqual),
                        (">", TokenKind::Greater),
                        '=',
                    ))
                    .into();
                }
                Some('<') => {
                    return Ok(self.consume_two(
                        ("<=", TokenKind::LessEqual),
                        ("<", TokenKind::Less),
                        '=',
                    ))
                    .into();
                }
                Some('=') => {
                    return Ok(self.consume_two(
                        ("==", TokenKind::EqualEqual),
                        ("=", TokenKind::Equal),
                        '=',
                    ))
                    .into();
                }
                Some('!') => {
                    return Ok(self.consume_two(
                        ("!=", TokenKind::BangEqual),
                        ("!", TokenKind::Bang),
                        '=',
                    ))
                    .into();
                }

                Some('"') => return self.consume_string().into(),
                Some('\n') => self.consume_new_line(),
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some(c) if c.is_ascii_digit() => return self.consume_number().into(),
                Some(c) if is_alpha(*c) => return self.consume_identifier().into(),
                Some(c) => {
                    let c = *c;
                    self.advance();
                    return Some(Err(ScanningError::LexicalError(self.line, c)));
                }
                None => {
                    self.end = true;
                    return Some(Ok(Token {
                        token: TokenKind::EOF,
                        lexeme: "",
                        value: TokenValue::Null,
                        line: self.line,
                    }));
                }
            }
        }
    }
}

pub fn tokenize(value: &str) -> Vec<Result<Token<'_>, ScanningError>> {
    let lexer = Lexer::new(value);

    return lexer.into_iter().collect();
}

pub mod prelude {
    pub use super::ScanningError;
    pub use super::Token;
    pub use super::tokenize;
}
