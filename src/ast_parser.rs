use std::{collections::HashSet, fmt::Display, iter::Peekable};

use crate::{
    Token, ast_parser,
    expressions::{
        Expression,
        group::{self, Group},
        unary_expression::{UnaryExpression, UnaryOp},
    },
    scanner::{Keyword, TokenKind},
};

pub struct LocalScope {
    idents: HashSet<String>,
}

impl LocalScope {
    pub fn new() -> Self {
        Self {
            idents: HashSet::new(),
        }
    }
}

impl Default for LocalScope {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AstParserState<'a> {
    pub pos: usize,
    pub tokens: &'a [Token<'a>],
    pub it: Peekable<std::slice::Iter<'a, Token<'a>>>,
}

pub enum AstParserScope {
    GlobalScope,
    LocalScope(Box<AstParserScope>, LocalScope),
    FunctionBody(Box<AstParserScope>, LocalScope),
    ClassBody(Box<AstParserScope>, LocalScope),
    DerivedClassBody(Box<AstParserScope>, LocalScope),
    MethodBody(Box<AstParserScope>, LocalScope),
}

impl AstParserScope {
    pub fn get_parent_scope(&self) -> Option<&AstParserScope> {
        match self {
            AstParserScope::GlobalScope => return None,
            AstParserScope::LocalScope(ast_parser, local_scope) => Some(ast_parser),
            AstParserScope::FunctionBody(ast_parser, local_scope) => Some(ast_parser),
            AstParserScope::ClassBody(ast_parser, local_scope) => Some(ast_parser),
            AstParserScope::DerivedClassBody(ast_parser, local_scope) => Some(ast_parser),
            AstParserScope::MethodBody(ast_parser, local_scope) => Some(ast_parser),
        }
    }

    pub fn into_parent(self) -> Option<AstParserScope> {
        match self {
            AstParserScope::GlobalScope => return None,
            AstParserScope::LocalScope(ast_parser, local_scope) => Some(*ast_parser),
            AstParserScope::FunctionBody(ast_parser, local_scope) => Some(*ast_parser),
            AstParserScope::ClassBody(ast_parser, local_scope) => Some(*ast_parser),
            AstParserScope::DerivedClassBody(ast_parser, local_scope) => Some(*ast_parser),
            AstParserScope::MethodBody(ast_parser, local_scope) => Some(*ast_parser),
        }
    }

    pub fn get_local_scope(&mut self) -> Option<&mut LocalScope> {
        match self {
            AstParserScope::GlobalScope => return None,
            AstParserScope::LocalScope(ast_parser, local_scope) => Some(local_scope),
            AstParserScope::FunctionBody(ast_parser, local_scope) => Some(local_scope),
            AstParserScope::ClassBody(ast_parser, local_scope) => Some(local_scope),
            AstParserScope::DerivedClassBody(ast_parser, local_scope) => Some(local_scope),
            AstParserScope::MethodBody(ast_parser, local_scope) => Some(local_scope),
        }
    }

    pub fn is_in_global_scope(&self) -> bool {
        return matches!(self, AstParserScope::GlobalScope);
    }

    pub fn is_in_function_scope(&self) -> bool {
        return matches!(
            self,
            AstParserScope::FunctionBody(_, _) | AstParserScope::MethodBody(_, _)
        ) || match self.get_parent_scope() {
            Some(v) => v.is_in_function_scope(),
            None => false,
        };
    }

    pub fn is_in_method_scope(&self) -> bool {
        return matches!(self, AstParserScope::MethodBody(_, _))
            || match self.get_parent_scope() {
                Some(v) => v.is_in_method_scope(),
                None => false,
            };
    }

    pub fn is_in_class_scope(&self) -> bool {
        return matches!(
            self,
            AstParserScope::ClassBody(_, _) | AstParserScope::DerivedClassBody(_, _)
        ) || match self.get_parent_scope() {
            Some(v) => v.is_in_class_scope(),
            None => false,
        };
    }

    pub fn is_in_derived_class_scope(&self) -> bool {
        return matches!(self, AstParserScope::DerivedClassBody(_, _))
            || match self.get_parent_scope() {
                Some(v) => v.is_in_derived_class_scope(),
                None => false,
            };
    }
}

pub struct AstParser<'a> {
    scope: AstParserScope,
    state: AstParserState<'a>,
}

#[derive(thiserror::Error, Debug)]
pub struct ParserError {
    error: ParserErrorDetails,
    line: usize,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.error)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParserErrorDetails {
    #[error("Tried to redefine variable {0} in local scope")]
    RedefinedVariableInLocalScope(String),
    #[error("Expected token, got EOF")]
    UnexpectedEof,
    #[error("Unexpecpected token {0}, expected {1}")]
    UnexpectedToken(TokenKind, String),
    #[error("Unexpecpected token {0}, expected {1}")]
    InvalidToken(TokenKind, TokenKind),
}

impl<'a> AstParser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            state: AstParserState {
                it: tokens.iter().peekable(),
                pos: 0,
                tokens: tokens,
            },
            scope: AstParserScope::GlobalScope,
        }
    }

    pub fn get_parent_scope(&self) -> Option<&AstParserScope> {
        return self.scope.get_parent_scope();
    }

    pub fn get_local_scope(&mut self) -> Option<&mut LocalScope> {
        return self.scope.get_local_scope();
    }

    pub fn is_in_global_scope(&self) -> bool {
        return self.scope.is_in_global_scope();
    }

    pub fn is_in_function_scope(&'a self) -> bool {
        return self.scope.is_in_function_scope();
    }

    pub fn is_in_method_scope(&'a self) -> bool {
        return self.scope.is_in_method_scope();
    }

    pub fn is_in_class_scope(&'a self) -> bool {
        return self.scope.is_in_class_scope();
    }

    pub fn is_in_derived_class_scope(&'a self) -> bool {
        return self.scope.is_in_derived_class_scope();
    }

    pub fn local_scope(self) -> Self {
        Self {
            scope: AstParserScope::LocalScope(Box::new(self.scope), LocalScope::new()),
            state: self.state,
        }
    }

    pub fn pop_scope(self) -> Self {
        match self.scope.into_parent() {
            Some(v) => Self {
                scope: v,
                state: self.state,
            },
            None => panic!("Tried to pop a scope that was already at global scope."),
        }
    }

    pub fn peek(&mut self) -> Option<&&Token<'_>> {
        self.state.it.peek()
    }

    pub fn advance(&mut self) -> &'a Token<'a> {
        let token = self.state.it.next();
        self.state.pos += 1;

        token.unwrap()
    }

    pub fn consume(&mut self, token: TokenKind) -> Result<(), ParserError> {
        if matches!(self.token_kind(), token) {
            self.advance();
            return Ok(());
        } else {
            let t = self.token_kind();
            self.error(ParserErrorDetails::InvalidToken(t, token))
        }
    }

    pub fn peek_or_last(&mut self) -> &Token<'a> {
        return self
            .state
            .it
            .peek()
            .unwrap_or(self.state.tokens.last().as_ref().unwrap());
    }

    pub fn line_number(&mut self) -> usize {
        self.peek_or_last().line
    }

    pub fn token_kind(&mut self) -> TokenKind {
        self.peek_or_last().token
    }

    pub fn unexpected_eof(&mut self) -> Result<(), ParserError> {
        if matches!(self.token_kind(), TokenKind::EOF) {
            return self.error(ParserErrorDetails::UnexpectedEof);
        }
        Ok(())
    }

    pub fn error<T>(&mut self, error: ParserErrorDetails) -> Result<T, ParserError> {
        return Err(ParserError {
            error,
            line: self.line_number(),
        });
    }

    pub fn define_identifier(&mut self, s: &str) -> Result<(), ParserError> {
        match self.get_local_scope() {
            Some(scope) => {
                if scope.idents.contains(s) {
                    return self.error(ParserErrorDetails::RedefinedVariableInLocalScope(
                        s.to_string(),
                    ));
                } else {
                    scope.idents.insert(s.to_string());
                    Ok(())
                }
            }
            None => return Ok(()),
        }
    }

    pub fn expression(&mut self) -> Result<Box<dyn Expression + 'a>, ParserError> {
        self.unary()
    }

    pub fn unary(&mut self) -> Result<Box<dyn Expression + 'a>, ParserError> {
        self.unexpected_eof()?;
        use crate::expressions::literal::Literal;
        match self.token_kind() {
            TokenKind::Bang => {
                self.advance();
                Ok(Box::new(UnaryExpression::new(UnaryOp::Bang, self.unary()?)))
            }
            TokenKind::Minus => {
                self.advance();
                Ok(Box::new(UnaryExpression::new(
                    UnaryOp::Minus,
                    self.unary()?,
                )))
            }
            _ => self.primary(),
        }
    }

    pub fn primary(&mut self) -> Result<Box<dyn Expression + 'a>, ParserError> {
        self.unexpected_eof()?;
        use crate::expressions::literal::Literal;
        match self.token_kind() {
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen);
                Ok(Box::new(Group::new(expr)))
            }
            TokenKind::Number => Ok(Box::new(Literal::new(self.advance()))),
            TokenKind::Keyword(Keyword::True) => Ok(Box::new(Literal::new(self.advance()))),
            TokenKind::String => Ok(Box::new(Literal::new(self.advance()))),
            TokenKind::Identifier => todo!(),
            TokenKind::Keyword(Keyword::False) => Ok(Box::new(Literal::new(self.advance()))),
            TokenKind::Keyword(Keyword::Nil) => Ok(Box::new(Literal::new(self.advance()))),

            c => self.error(ParserErrorDetails::UnexpectedToken(
                c,
                "primary".to_string(),
            )),
        }
    }
}

pub mod prelude {
    pub use super::AstParser;
    pub use super::ParserError;
}
