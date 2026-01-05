use std::{collections::HashSet, fmt::Display, iter::Peekable};

use crate::{
    Token, ast_parser,
    expressions::{
        Expression, Expressions,
        assignment_expression::AssignmentExpression,
        binary_expression::{BinaryExpression, BinaryOp},
        equality_expression::{EqualityExpression, EqualityOp},
        group::{self, Group},
        identifier::Identifier,
        relation_expression::{RelationalExpression, RelationalOp},
        unary_expression::{UnaryExpression, UnaryOp},
    },
    scanner::{Keyword, TokenKind},
    statements::{
        Statement,
        declare_statement::DeclareStatement,
        expression_statement::ExprStatement,
        print_statement::{self, PrintStatement},
    },
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
    #[error("Invalid assignment target")]
    InvalidAssignementTarget,
}

impl<'a> AstParser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            state: AstParserState {
                it: tokens.iter().peekable(),
                pos: 0,
                tokens: tokens,
            },
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
        if self.token_kind() == token {
            self.advance();
            return Ok(());
        } else {
            let t = self.token_kind();
            self.error(ParserErrorDetails::InvalidToken(t, token))
        }
    }

    pub fn peek_or_last(&mut self) -> &'a Token<'a> {
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

    pub fn expression(&mut self) -> Result<Expressions<'a>, ParserError> {
        self.assignment()
    }

    pub fn assignment(&mut self) -> Result<Expressions<'a>, ParserError> {
        let expr = self.equality()?;
        if self.token_kind() == TokenKind::Equal {
            self.advance();
            let right = self.assignment()?;

            match expr {
                Expressions::Identifier(ident) => {
                    return Ok(AssignmentExpression::new(ident, Box::new(right)).into());
                }
                _ => {
                    return self.error(ParserErrorDetails::InvalidAssignementTarget);
                }
            }
        }

        return Ok(expr);
    }

    pub fn equality(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut rel = self.relational()?;

        while matches!(
            self.token_kind(),
            TokenKind::EqualEqual | TokenKind::BangEqual
        ) {
            let op = match self.token_kind() {
                TokenKind::EqualEqual => EqualityOp::EqualEqual,
                TokenKind::BangEqual => EqualityOp::BangEqual,
                _ => unreachable!(),
            };

            self.advance();

            let right = Box::new(self.relational()?);
            rel = EqualityExpression::new(op, Box::new(rel), right).into();
        }

        Ok(rel)
    }

    pub fn relational(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut term = self.term()?;

        while matches!(
            self.token_kind(),
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual
        ) {
            let op = match self.token_kind() {
                TokenKind::Less => RelationalOp::Less,
                TokenKind::LessEqual => RelationalOp::LessEqual,
                TokenKind::Greater => RelationalOp::Greater,
                TokenKind::GreaterEqual => RelationalOp::GreaterEqual,
                _ => unreachable!(),
            };

            self.advance();

            let right = self.term()?;
            term = RelationalExpression::new(op, Box::new(term), Box::new(right)).into();
        }

        Ok(term)
    }

    pub fn term(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut factor = self.factor()?;

        while matches!(self.token_kind(), TokenKind::Plus | TokenKind::Minus) {
            let op = match self.token_kind() {
                TokenKind::Plus => BinaryOp::Plus,
                TokenKind::Minus => BinaryOp::Minus,
                _ => unreachable!(),
            };

            self.advance();

            let right = self.factor()?;
            factor = BinaryExpression::new(op, Box::new(factor), Box::new(right)).into();
        }

        Ok(factor)
    }

    pub fn factor(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut unary = self.unary()?;

        while matches!(self.token_kind(), TokenKind::Slash | TokenKind::Star) {
            let op = match self.token_kind() {
                TokenKind::Star => BinaryOp::Star,
                TokenKind::Slash => BinaryOp::Slash,
                _ => unreachable!(),
            };

            self.advance();

            let right = self.unary()?;
            unary = BinaryExpression::new(op, Box::new(unary), Box::new(right)).into();
        }

        Ok(unary)
    }

    pub fn unary(&mut self) -> Result<Expressions<'a>, ParserError> {
        self.unexpected_eof()?;
        use crate::expressions::literal::Literal;
        match self.token_kind() {
            TokenKind::Bang => {
                self.advance();
                Ok(UnaryExpression::new(UnaryOp::Bang, Box::new(self.unary()?)).into())
            }
            TokenKind::Minus => {
                self.advance();
                Ok(UnaryExpression::new(UnaryOp::Minus, Box::new(self.unary()?)).into())
            }
            _ => self.primary(),
        }
    }

    pub fn identifier(&mut self) -> Result<Identifier<'a>, ParserError> {
        match self.token_kind() {
            TokenKind::Identifier => Ok(Identifier::new(self.advance())),
            c => self.error(ParserErrorDetails::UnexpectedToken(
                c,
                "identifier".to_string(),
            )),
        }
    }

    pub fn primary(&mut self) -> Result<Expressions<'a>, ParserError> {
        self.unexpected_eof()?;
        use crate::expressions::literal::Literal;
        match self.token_kind() {
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen)?;
                Ok(Group::new(Box::new(expr)).into())
            }
            TokenKind::Number => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::True) => Ok((Literal::new(self.advance())).into()),
            TokenKind::String => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::False) => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::Nil) => Ok((Literal::new(self.advance())).into()),
            TokenKind::Identifier => Ok(self.identifier()?.into()),
            c => self.error(ParserErrorDetails::UnexpectedToken(
                c,
                "primary".to_string(),
            )),
        }
    }
    pub fn print_statement(&mut self) -> Result<Box<PrintStatement<'a>>, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon)?;
        let statement = PrintStatement::new(expr);
        Ok(Box::new(statement))
    }

    pub fn expr_statement(&mut self) -> Result<Box<ExprStatement<'a>>, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon)?;
        let statement = ExprStatement::new(expr);
        Ok(Box::new(statement))
    }

    pub fn variable_declaration(&mut self) -> Result<Box<DeclareStatement<'a>>, ParserError> {
        let ident = self.identifier()?;
        let value = if self.token_kind() == TokenKind::Equal {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon)?;

        let statement = DeclareStatement::new(ident, value);

        return Ok(Box::new(statement));
    }

    pub fn declaration(&mut self) -> Result<Box<dyn Statement + 'a>, ParserError> {
        match self.token_kind() {
            TokenKind::Keyword(Keyword::Var) => {
                self.advance();
                self.variable_declaration().map(|i| i as Box<dyn Statement>)
            }
            _ => self.statement(),
        }
    }

    pub fn statement(&mut self) -> Result<Box<dyn Statement + 'a>, ParserError> {
        match self.token_kind() {
            TokenKind::Keyword(Keyword::Print) => {
                self.advance();
                self.print_statement().map(|i| i as Box<dyn Statement>)
            }
            _ => self.expr_statement().map(|i| i as Box<dyn Statement>),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<dyn Statement + 'a>>, ParserError> {
        let mut result = vec![];
        while !matches!(self.token_kind(), TokenKind::EOF) {
            result.push(self.declaration()?);
        }
        Ok(result)
    }
}

pub mod prelude {
    pub use super::AstParser;
    pub use super::ParserError;
}
