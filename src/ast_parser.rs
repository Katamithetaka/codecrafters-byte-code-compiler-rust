use std::{fmt::Display, iter::Peekable};

use crate::{
    Token,
    expressions::{
        Expressions, assignment_expression::AssignmentExpression, binary_expression::{BinaryExpression, BinaryOp}, call_expression::CallExpression, equality_expression::{EqualityExpression, EqualityOp}, group::Group, identifier::Identifier, logical_expression::{LogicalExpression, LogicalOp}, relation_expression::{RelationalExpression, RelationalOp}, unary_expression::{UnaryExpression, UnaryOp}
    },
    scanner::{Keyword, TokenKind},
    statements::{
        Statements, block_statement::BlockStatement, declare_statement::DeclareStatement, expression_statement::ExprStatement, for_statement::ForStatement, function_declaration_statement::FunctionDeclareStatement, if_statement::IfStatement, print_statement::PrintStatement, return_statement::ReturnStatement, while_statements::WhileStatement
    },
};

pub struct AstParserState<'a> {
    pub pos: usize,
    pub tokens: &'a [Token<'a>],
    pub it: Peekable<std::slice::Iter<'a, Token<'a>>>,
}

pub struct AstParser<'a> {
    state: AstParserState<'a>,
}

#[derive(thiserror::Error, Debug)]
pub struct ParserError {
    pub error: ParserErrorDetails,
    pub line: usize,
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
        while self.token_kind() == TokenKind::LeftParen {
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
        let statement = ReturnStatement::new(expr, self.line_number());
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
    
    pub fn function_declaration(&mut self) -> Result<Statements<'a>, ParserError> {
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
        
        let statement = FunctionDeclareStatement::new(fun_name, args, statements);
        
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

pub mod prelude {
    pub use super::AstParser;
    pub use super::ParserError;
}
