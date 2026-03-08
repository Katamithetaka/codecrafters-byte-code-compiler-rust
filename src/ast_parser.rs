use std::{fmt::Display, iter::Peekable};

use crate::{
    Token, compiler::int_types::line_type, expressions::{
        Expressions, assignment_expression::AssignmentExpression, binary_expression::{BinaryExpression, BinaryOp}, call_expression::CallExpression, equality_expression::{EqualityExpression, EqualityOp}, get_expression::GetExpression, group::Group, identifier::Identifier, logical_expression::{LogicalExpression, LogicalOp}, relation_expression::{RelationalExpression, RelationalOp}, set_expression::SetExpression, unary_expression::{UnaryExpression, UnaryOp}
    }, prelude::ClassDeclareStatement, scanner::{Keyword, TokenKind, TokenValue}, statements::{
        Statements, block_statement::BlockStatement, declare_statement::DeclareStatement, expression_statement::ExprStatement, for_statement::ForStatement, function_declaration_statement::FunctionDeclareStatement, if_statement::IfStatement, print_statement::PrintStatement, return_statement::ReturnStatement, while_statements::WhileStatement
    }, value::{Function, Value, callable::FunctionKind}
};

/// Represents the state of the AST parser, including the current position,
/// the list of tokens being parsed, and an iterator over the tokens.
pub struct AstParserState<'a> {
    /// The current position in the token stream.
    pub pos: usize,
    /// The list of tokens to be parsed.
    pub tokens: &'a [Token<'a>],
    /// A peekable iterator over the tokens.
    pub it: Peekable<std::slice::Iter<'a, Token<'a>>>,
}

/// The main AST parser that processes tokens and generates an abstract syntax tree (AST).
pub struct AstParser<'a> {
    /// The internal state of the parser.
    state: AstParserState<'a>,
}

#[derive(thiserror::Error, Debug)]
/// Represents an error encountered during parsing.
///
/// This struct contains details about the error and the line number where it occurred.
pub struct ParserError {
    /// The specific details of the parsing error.
    pub error: ParserErrorDetails,
    /// The line number where the error occurred.
    pub line: line_type,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.error)
    }
}

/// Enum representing the various types of errors that can occur during parsing.
#[derive(thiserror::Error, Debug)]
pub enum ParserErrorDetails {
    /// Error for attempting to redefine a variable in the local scope.
    #[error("Tried to redefine variable {0} in local scope")]
    RedefinedVariableInLocalScope(String),
    /// Error for encountering an unexpected end of file.
    #[error("Expected token, got EOF")]
    UnexpectedEof,
    /// Error for encountering an unexpected token.
    #[error("Unexpecpected token {0}, expected {1}")]
    UnexpectedToken(TokenKind, String),
    /// Error for encountering an invalid token.
    #[error("Unexpecpected token {0}, expected {1}")]
    InvalidToken(TokenKind, TokenKind),
    /// Error for an invalid assignment target.
    #[error("Invalid assignment target")]
    InvalidAssignementTarget,
    #[error("Variable redeclaration")]
    VariableRedeclaration,
    #[error("Return statement in top-level code")]
    InvalidReturnStatement
}

impl<'a> AstParser<'a> {
    /// Creates a new instance of the AST parser with the given tokens.
    ///
    /// # Arguments
    ///
    /// * `tokens` - A slice of tokens to be parsed.
    ///
    /// # Returns
    ///
    /// A new `AstParser` instance.
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            state: AstParserState {
                it: tokens.iter().peekable(),
                pos: 0,
                tokens: tokens,
            },
        }
    }

    /// Returns a reference to the next token in the stream without consuming it.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the next token, or `None` if the end of the stream is reached.
    pub fn peek(&mut self) -> Option<&&Token<'_>> {
        self.state.it.peek()
    }

    /// Advances the parser to the next token in the stream.
    ///
    /// # Returns
    ///
    /// A reference to the current token.
    pub fn advance(&mut self) -> &'a Token<'a> {
        let token = self.state.it.next();
        self.state.pos += 1;

        token.unwrap()
    }

    /// Consumes the current token if it matches the expected token kind.
    ///
    /// # Arguments
    ///
    /// * `token` - The expected token kind.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the token matches the expected kind.
    /// * `Err(ParserError)` if the token does not match the expected kind.
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

    pub fn line_number(&mut self) -> line_type {
        self.peek_or_last().line as line_type
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
            line: self.line_number() as line_type,
        });
    }

    pub fn expression(&mut self) -> Result<Expressions<'a>, ParserError> {
        self.assignment()
    }

    pub fn or(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut expr = self.and()?;

        while self.token_kind() == TokenKind::Keyword(Keyword::Or) {
            self.advance();

            let right = self.and()?;
            expr = LogicalExpression::new(LogicalOp::Or, Box::new(expr), Box::new(right)).into();
        }

        Ok(expr)
    }

    pub fn and(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut expr = self.equality()?;

        while self.token_kind() == TokenKind::Keyword(Keyword::And) {
            self.advance();

            let right = self.equality()?;
            expr = LogicalExpression::new(LogicalOp::And, Box::new(expr), Box::new(right)).into();
        }

        Ok(expr)
    }

    pub fn assignment(&mut self) -> Result<Expressions<'a>, ParserError> {
        let expr = self.or()?;
        if self.token_kind() == TokenKind::Equal {
            self.advance();
            let right = self.assignment()?;

            match expr {
                Expressions::Identifier(ident) => {
                    return Ok(AssignmentExpression::new(ident, Box::new(right)).into());
                }
                Expressions::GetExpression(expression) => {
                    return Ok(SetExpression::new(expression, Box::new(right)).into());
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

    pub fn call(&mut self) -> Result<Expressions<'a>, ParserError> {
        let mut expr = self.primary()?;
        loop {
            if self.token_kind() == TokenKind::LeftParen {
                self.advance();
                let mut arguments = vec![];
                while self.token_kind() != TokenKind::RightParen {
                    arguments.push(self.expression()?);
                    if self.token_kind() != TokenKind::RightParen {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                self.consume(TokenKind::RightParen)?;
                expr = CallExpression::new(Box::new(expr), arguments).into();
            }
            else if self.token_kind() == TokenKind::Dot {
                self.advance();
                let identifier = self.identifier()?;
                expr = GetExpression::new(Box::new(expr), identifier).into();

            }
            else {
                break;
            }
        };

        Ok(expr.into())

    }

    pub fn unary(&mut self) -> Result<Expressions<'a>, ParserError> {
        self.unexpected_eof()?;
        match self.token_kind() {
            TokenKind::Bang => {
                self.advance();
                Ok(UnaryExpression::new(UnaryOp::Bang, Box::new(self.unary()?)).into())
            }
            TokenKind::Minus => {
                self.advance();
                Ok(UnaryExpression::new(UnaryOp::Minus, Box::new(self.unary()?)).into())
            }
            _ => self.call(),
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

    pub fn group(&mut self) -> Result<Expressions<'a>, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenKind::RightParen)?;
        Ok(Group::new(Box::new(expr)).into())
    }

    pub fn primary(&mut self) -> Result<Expressions<'a>, ParserError> {
        self.unexpected_eof()?;
        use crate::expressions::literal::Literal;
        match self.token_kind() {
            TokenKind::LeftParen => {
                self.advance();
                self.group()
            }
            TokenKind::Number => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::True) => Ok((Literal::new(self.advance())).into()),
            TokenKind::String => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::False) => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::Nil) => Ok((Literal::new(self.advance())).into()),
            TokenKind::Keyword(Keyword::This) => {
                self.advance();
                Ok(Expressions::Identifier(Identifier {
                    token: "this",
                    line: self.line_number(),
                }))
            }
            TokenKind::Identifier => Ok(self.identifier()?.into()),
            c => self.error(ParserErrorDetails::UnexpectedToken(
                c,
                "primary".to_string(),
            )),
        }
    }
    pub fn print_statement(&mut self) -> Result<PrintStatement<'a>, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon)?;
        let statement = PrintStatement::new(expr);
        Ok(statement)
    }

    pub fn return_statement(&mut self) -> Result<ReturnStatement<'a>, ParserError> {
        let expr = if self.token_kind() != TokenKind::Semicolon {
            let expr = self.expression()?;
            Some(expr)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon)?;
        let statement = ReturnStatement::new(expr, self.line_number() as line_type);
        Ok(statement)
    }

    pub fn expr_statement(&mut self) -> Result<ExprStatement<'a>, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon)?;
        let statement = ExprStatement::new(expr);
        Ok(statement)
    }

    pub fn variable_declaration(&mut self) -> Result<DeclareStatement<'a>, ParserError> {
        let ident = self.identifier()?;
        let value = if self.token_kind() == TokenKind::Equal {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon)?;

        let statement = DeclareStatement::new(ident, value);

        return Ok(statement);
    }

    pub fn function_declaration(&mut self) -> Result<FunctionDeclareStatement<'a>, ParserError> {
        let fun_name = self.identifier()?;
        self.consume(TokenKind::LeftParen)?;
        let mut args = vec![];
        while self.token_kind() != TokenKind::RightParen {
            args.push(self.identifier()?);
            if self.token_kind() != TokenKind::Comma {
                break;
            }
            self.advance();
        }

        self.consume(TokenKind::RightParen)?;

        let mut statements = vec![];
        self.consume(TokenKind::LeftBrace)?;
        while self.token_kind() != TokenKind::RightBrace {
            statements.push(self.declaration()?);
        }
        self.consume(TokenKind::RightBrace)?;

        let statement = FunctionDeclareStatement::new(fun_name, args, statements, FunctionKind::Function);

        return Ok(statement.into())
    }

    pub fn class_declaration(&mut self) -> Result<Statements<'a>, ParserError> {
        let class_name = self.identifier()?;
        self.consume(TokenKind::LeftBrace)?;
        let mut functions = vec![];
        while self.token_kind() != TokenKind::RightBrace {
            functions.push(self.function_declaration()?);
        }
        self.consume(TokenKind::RightBrace)?;

        let statement = ClassDeclareStatement::new(class_name, functions);

        return Ok(statement.into())
    }

    pub fn declaration(&mut self) -> Result<Statements<'a>, ParserError> {

        match self.token_kind() {
            TokenKind::Keyword(Keyword::Var) => {
                self.advance();
                Ok(self.variable_declaration()?.into())
            }
            TokenKind::Keyword(Keyword::Fun) => {
                self.advance();
                Ok(self.function_declaration()?.into())
            }
            TokenKind::Keyword(Keyword::Class) => {
                self.advance();
                Ok(self.class_declaration()?.into())
            }
            _ => self.statement(),
        }
    }

    pub fn block_statement(&mut self) -> Result<BlockStatement<'a>, ParserError> {
        let mut statements = vec![];
        let begin_line = self.line_number();
        while self.token_kind() != TokenKind::RightBrace && self.token_kind() != TokenKind::EOF {
            statements.push(self.declaration()?);
        }
        let end_line = self.line_number();

        self.consume(TokenKind::RightBrace)?;

        Ok(BlockStatement::new(statements, begin_line, end_line))
    }

    pub fn if_statement(&mut self) -> Result<IfStatement<'a>, ParserError> {
        self.consume(TokenKind::LeftParen)?;
        let expression = self.group()?;
        let statement = self.statement()?;
        let mut statements = vec![(Some(expression), statement, self.line_number())];

        while self.token_kind() == TokenKind::Keyword(Keyword::Else) {
            self.advance();
            if self.token_kind() == TokenKind::Keyword(Keyword::If) {
                self.advance();
                self.consume(TokenKind::LeftParen)?;
                statements.push((Some(self.group()?), self.statement()?, self.line_number()));
            } else {
                statements.push((None, self.statement()?, self.line_number()));
            }
        }

        return Ok(IfStatement::new(statements));
    }

    pub fn while_statement(&mut self) -> Result<Statements<'a>, ParserError> {
        self.consume(TokenKind::LeftParen)?;
        let expression = self.group()?;
        let statement = self.statement()?;
        return Ok(WhileStatement::new(expression, Box::new(statement)).into());
    }

    pub fn for_statement(&mut self) -> Result<Statements<'a>, ParserError> {
        self.consume(TokenKind::LeftParen)?;
        let begin_line = self.line_number();
        let declaration: Option<Box<Statements<'a>>> =
        if self.token_kind() != TokenKind::Semicolon {
            if self.token_kind() == TokenKind::Keyword(Keyword::Var) {
                self.advance();
                Some(Box::new(self.variable_declaration()?.into()))
            }
            else {
               Some(Box::new(self.expr_statement()?.into()))
            }
        }
        else {
            self.advance();
            None
        };
        let test: Option<Expressions<'a>> =
            if self.token_kind() != TokenKind::Semicolon {
                Some(self.expression()?.into())
            }
            else {
                None
            };
        self.consume(TokenKind::Semicolon)?;

        let inc: Option<Expressions<'a>> =
            if self.token_kind() != TokenKind::RightParen {
                Some(self.expression()?.into())
            }
            else {
                None
            };

        self.consume(TokenKind::RightParen)?;

        let statement = Box::new(self.statement()?);
        let end_line = self.line_number();

        return Ok(ForStatement::new(declaration, test, inc, statement, begin_line, end_line).into())
    }


    pub fn statement(&mut self) -> Result<Statements<'a>, ParserError> {
        match self.token_kind() {
            TokenKind::Keyword(Keyword::Print) => {
                self.advance();
                Ok(self.print_statement()?.into())
            }
            TokenKind::Keyword(Keyword::If) => {
                self.advance();
                Ok(self.if_statement()?.into())
            }
            TokenKind::Keyword(Keyword::While) => {
                self.advance();
                Ok(self.while_statement()?.into())
            }
            TokenKind::Keyword(Keyword::For) => {
                self.advance();
                Ok(self.for_statement()?.into())
            }
            TokenKind::Keyword(Keyword::Return) => {
                self.advance();
                Ok(self.return_statement()?.into())
            }
            TokenKind::LeftBrace => {
                self.advance();
                Ok(self.block_statement()?.into())
            }
            _ => Ok(self.expr_statement()?.into()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statements<'a>>, ParserError> {
        let mut result = vec![];
        while !matches!(self.token_kind(), TokenKind::EOF) {
            result.push(self.declaration()?);
        }
        Ok(result)
    }
}

/// The `prelude` module re-exports commonly used types and functions from this module.
///
/// This allows for easier imports in other parts of the codebase.
pub mod prelude {
    pub use super::AstParser;
    pub use super::ParserError;
}
