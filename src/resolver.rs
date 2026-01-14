use std::{
    collections::{HashMap},
};

use crate::{
    ParserError,
    ast_parser::ParserErrorDetails,
    expressions::{
        Expressions, assignment_expression::AssignmentExpression, binary_expression::BinaryExpression, call_expression::CallExpression, equality_expression::EqualityExpression, group::Group, identifier::{Identifier, IdentifierKind}, logical_expression::LogicalExpression, relation_expression::RelationalExpression, unary_expression::UnaryExpression
    },
    statements::{
        Statements, block_statement::BlockStatement, declare_statement::DeclareStatement, expression_statement::ExprStatement, for_statement::ForStatement, function_declaration_statement::FunctionDeclareStatement, if_statement::IfStatement, print_statement::PrintStatement, return_statement::ReturnStatement, while_statements::WhileStatement
    },
};

/// Represents a local scope in the resolver, containing identifiers and their associated stack indices.
pub struct LocalScope {
    /// A map of identifier names to their stack indices.
    idents: HashMap<String, u16>,
}

/// Implementation of methods for `LocalScope`.
impl LocalScope {
    /// Creates a new, empty `LocalScope`.
    ///
    /// # Returns
    ///
    /// A new instance of `LocalScope` with an empty identifier map.
    pub fn new() -> Self {
        Self {
            idents: HashMap::new(),
        }
    }

    pub fn declare_identifier(
        &mut self,
        name: &mut Identifier,
        stack_index: u16,
    ) -> Result<(), ParserError> {
        if self.idents.contains_key(name.token) {
            return Err(ParserError {
                error: ParserErrorDetails::RedefinedVariableInLocalScope(name.token.to_string()),
                line: name.line,
            });
        }
        name.kind = IdentifierKind::LocalScope { slot: stack_index };

        self.idents.insert(name.token.to_string(), stack_index);

        Ok(())
    }
}

impl Default for LocalScope {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the different types of scopes in the resolver.
pub enum ResolverScope {
    /// The global scope, containing global variables and functions.
    GlobalScope,
    /// A local scope nested within another scope.
    LocalScope(Box<ResolverScope>, LocalScope),
    /// A function body scope, containing local variables and parameters.
    FunctionBody(Box<ResolverScope>, LocalScope),
    /// A class body scope, containing class-level variables and methods.
    ClassBody(Box<ResolverScope>, LocalScope),
    /// A derived class body scope, inheriting from another class.
    DerivedClassBody(Box<ResolverScope>, LocalScope),
    /// A method body scope, containing method-specific variables.
    MethodBody(Box<ResolverScope>, LocalScope),
}

/// Represents the kind of a local identifier in the resolver.
pub enum LocalIdentifierKind {
    /// An identifier defined in the current local scope.
    LocalScope,
    /// An identifier defined in an upper (enclosing) scope.
    UpperScope,
}

impl ResolverScope {
    pub fn get_parent_scope(&self) -> Option<&ResolverScope> {
        match self {
            ResolverScope::GlobalScope => return None,
            ResolverScope::LocalScope(ast_parser, _) => Some(ast_parser),
            ResolverScope::FunctionBody(ast_parser, _) => Some(ast_parser),
            ResolverScope::ClassBody(ast_parser, _) => Some(ast_parser),
            ResolverScope::DerivedClassBody(ast_parser, _) => Some(ast_parser),
            ResolverScope::MethodBody(ast_parser, _) => Some(ast_parser),
        }
    }

    pub fn into_parent(self) -> Option<ResolverScope> {
        match self {
            ResolverScope::GlobalScope => return None,
            ResolverScope::LocalScope(ast_parser, _) => Some(*ast_parser),
            ResolverScope::FunctionBody(ast_parser, _) => Some(*ast_parser),
            ResolverScope::ClassBody(ast_parser, _) => Some(*ast_parser),
            ResolverScope::DerivedClassBody(ast_parser, _) => Some(*ast_parser),
            ResolverScope::MethodBody(ast_parser, _) => Some(*ast_parser),
        }
    }

    pub fn get_local_scope_mut(&mut self) -> Option<&mut LocalScope> {
        match self {
            ResolverScope::GlobalScope => return None,
            ResolverScope::LocalScope(_, local_scope) => Some(local_scope),
            ResolverScope::FunctionBody(_, local_scope) => Some(local_scope),
            ResolverScope::ClassBody(_, local_scope) => Some(local_scope),
            ResolverScope::DerivedClassBody(_, local_scope) => Some(local_scope),
            ResolverScope::MethodBody(_, local_scope) => Some(local_scope),
        }
    }

    pub fn get_local_scope(&self) -> Option<&LocalScope> {
        match self {
            ResolverScope::GlobalScope => return None,
            ResolverScope::LocalScope(_, local_scope) => Some(local_scope),
            ResolverScope::FunctionBody(_, local_scope) => Some(local_scope),
            ResolverScope::ClassBody(_, local_scope) => Some(local_scope),
            ResolverScope::DerivedClassBody(_, local_scope) => Some(local_scope),
            ResolverScope::MethodBody(_, local_scope) => Some(local_scope),
        }
    }

    pub fn is_in_global_scope(&self) -> bool {
        return matches!(self, ResolverScope::GlobalScope);
    }

    pub fn is_in_function_scope(&self) -> bool {
        return matches!(
            self,
            ResolverScope::FunctionBody(_, _) | ResolverScope::MethodBody(_, _)
        ) || match self.get_parent_scope() {
            Some(v) => v.is_in_function_scope(),
            None => false,
        };
    }

    pub fn is_in_method_scope(&self) -> bool {
        return matches!(self, ResolverScope::MethodBody(_, _))
            || match self.get_parent_scope() {
                Some(v) => v.is_in_method_scope(),
                None => false,
            };
    }

    pub fn is_in_class_scope(&self) -> bool {
        return matches!(
            self,
            ResolverScope::ClassBody(_, _) | ResolverScope::DerivedClassBody(_, _)
        ) || match self.get_parent_scope() {
            Some(v) => v.is_in_class_scope(),
            None => false,
        };
    }

    pub fn is_in_derived_class_scope(&self) -> bool {
        return matches!(self, ResolverScope::DerivedClassBody(_, _))
            || match self.get_parent_scope() {
                Some(v) => v.is_in_derived_class_scope(),
                None => false,
            };
    }

    pub fn get_local_identifier_kind(&self, name: &str) -> Option<LocalIdentifierKind> {
        match self.get_local_scope() {
            Some(scope) => scope
                .idents
                .contains_key(name)
                .then_some(LocalIdentifierKind::LocalScope)
                .or_else(|| {
                    self.get_parent_scope()
                        .map(|p| p.get_local_identifier_kind(name))
                        .flatten()
                        .map(|_| LocalIdentifierKind::UpperScope)
                }),
            None => None,
        }
    }

    pub fn get_slot_offset(&self) -> u8 {
        match self.get_parent_scope() {
            Some(parent) => match parent.get_local_scope() {
                Some(s) => s.idents.len() as u8 + parent.get_slot_offset(),
                None => 0,
            },
            None => 0,
        }
    }

    pub fn get_local_identifier_slot(&self, name: &str) -> Option<u16> {
        match self.get_local_scope() {
            Some(scope) => scope.idents.get(name).copied().or_else(|| {
                self.get_parent_scope()
                    .map(|p| p.get_local_identifier_slot(name))
                    .flatten()
            }),
            None => None,
        }
    }
}

/// The main resolver responsible for managing variable scopes and resolving identifiers.
pub struct Resolver {
    /// The current scope of the resolver.
    scope: ResolverScope,
    /// The current stack index for variable allocation.
    stack_index: u16,
    /// The stack of variable indices for nested scopes.
    stack: Vec<u16>,
}

/// The `prelude` module re-exports commonly used types from the resolver module.
///
/// This allows for easier imports in other parts of the codebase.
pub mod prelude {
    pub use super::{Resolver, ResolverScope, LocalScope, LocalIdentifierKind};
}

/// Implementation of methods for `Resolver`.
impl Resolver {
    /// Creates a new instance of the resolver with a global scope.
    ///
    /// # Returns
    ///
    /// A new `Resolver` instance with an empty stack and global scope.
    pub fn new() -> Self {
        Self {
            scope: ResolverScope::GlobalScope,
            stack_index: 0,
            stack: vec![],
        }
    }

    pub fn push_scope(&mut self) -> ResolverScope {
        let old_scope = std::mem::replace(
            &mut self.scope,
            ResolverScope::GlobalScope, // or some dummy value
        );

        self.stack.push(self.stack_index);

        old_scope
    }

    pub fn push_local_scope(&mut self) {
        let old_scope = self.push_scope();

        self.scope = ResolverScope::LocalScope(Box::new(old_scope), LocalScope::new());
    }

    pub fn push_function_scope(&mut self) {
        let old_scope = self.push_scope();

        self.scope = ResolverScope::FunctionBody(Box::new(old_scope), LocalScope::new());
    }

    pub fn push_class_scope(&mut self) {
        let old_scope = self.push_scope();
        self.scope = ResolverScope::ClassBody(Box::new(old_scope), LocalScope::new());
    }

    pub fn push_derived_class_scope(&mut self) {
        let old_scope = self.push_scope();

        self.scope = ResolverScope::DerivedClassBody(Box::new(old_scope), LocalScope::new());
    }

    pub fn push_method_scope(&mut self) {
        let old_scope = std::mem::replace(
            &mut self.scope,
            ResolverScope::GlobalScope, // or some dummy value
        );

        self.stack.push(self.stack_index);

        self.scope = ResolverScope::MethodBody(Box::new(old_scope), LocalScope::new());
    }

    pub fn pop_scope(&mut self) {
        let old_scope = std::mem::replace(
            &mut self.scope,
            ResolverScope::GlobalScope, // or some dummy value
        );

        self.scope = match old_scope.into_parent() {
            Some(s) => s,
            None => panic!("Tried to pop GlobalScope!"),
        };

        self.stack_index = self.stack.pop().unwrap();
    }

    pub fn resolve_binary<'a>(
        &mut self,
        binary_expression: BinaryExpression<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(BinaryExpression::new(
            binary_expression.op,
            Box::new(self.resolve_expr(*binary_expression.lhs)?),
            Box::new(self.resolve_expr(*binary_expression.rhs)?),
        )
        .into())
    }

    pub fn resolve_equality<'a>(
        &mut self,
        binary_expression: EqualityExpression<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(EqualityExpression::new(
            binary_expression.op,
            Box::new(self.resolve_expr(*binary_expression.lhs)?),
            Box::new(self.resolve_expr(*binary_expression.rhs)?),
        )
        .into())
    }

    pub fn resolve_relation<'a>(
        &mut self,
        binary_expression: RelationalExpression<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(RelationalExpression::new(
            binary_expression.op,
            Box::new(self.resolve_expr(*binary_expression.lhs)?),
            Box::new(self.resolve_expr(*binary_expression.rhs)?),
        )
        .into())
    }

    pub fn resolve_assignment<'a>(
        &mut self,
        binary_expression: AssignmentExpression<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(AssignmentExpression::new(
            self.resolve_identifier(binary_expression.lhs)?,
            Box::new(self.resolve_expr(*binary_expression.rhs)?),
        )
        .into())
    }

    pub fn resolve_logical<'a>(
        &mut self,
        binary_expression: LogicalExpression<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(LogicalExpression::new(
            binary_expression.op,
            Box::new(self.resolve_expr(*binary_expression.lhs)?),
            Box::new(self.resolve_expr(*binary_expression.rhs)?),
        )
        .into())
    }

    pub fn resolve_unary<'a>(
        &mut self,
        unary_expression: UnaryExpression<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(UnaryExpression::new(
            unary_expression.op,
            Box::new(self.resolve_expr(*unary_expression.rhs)?),
        )
        .into())
    }

    pub fn resolve_group<'a>(
        &mut self,
        unary_expression: Group<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        Ok(Group::new(Box::new(self.resolve_expr(*unary_expression.expr)?)).into())
    }

    pub fn resolve_identifier<'a>(
        &mut self,
        mut ident: Identifier<'a>,
    ) -> Result<Identifier<'a>, ParserError> {
        match self.scope.get_local_identifier_kind(ident.token) {
            Some(LocalIdentifierKind::UpperScope) => {
                if matches!(
                    self.scope,
                    ResolverScope::FunctionBody(_, _) | ResolverScope::MethodBody(_, _)
                ) {
                    ident.kind = IdentifierKind::UpperScope {
                        slot: self.scope.get_local_identifier_slot(ident.token).unwrap(),
                    };
                    return Ok(ident.into());
                } else {
                    ident.kind = IdentifierKind::LocalScope {
                        slot: self.scope.get_local_identifier_slot(ident.token).unwrap(),
                    };
                    return Ok(ident.into());
                }
            }
            Some(LocalIdentifierKind::LocalScope) => {
                ident.kind = IdentifierKind::LocalScope {
                    slot: self.scope.get_local_identifier_slot(ident.token).unwrap(),
                };
                return Ok(ident.into());
            }
            None => Ok(ident.into()),
        }
    }
    
    pub fn resolve_call<'a>(&mut self, mut expr: CallExpression<'a>) -> Result<Expressions<'a>, ParserError>  {
        expr.lhs = Box::new(self.resolve_expr(*expr.lhs)?);
        let  v = std::mem::replace(&mut expr.arguments, vec![]);
        for i in v.into_iter() {
            expr.arguments.push(self.resolve_expr(i)?);
        };
        
        Ok(expr.into())
    }

    pub fn resolve_expr<'a>(
        &mut self,
        expr: Expressions<'a>,
    ) -> Result<Expressions<'a>, ParserError> {
        match expr {
            Expressions::Identifier(ident) => Ok(self.resolve_identifier(ident)?.into()),
            Expressions::Group(expr) => self.resolve_group(expr),
            Expressions::BinaryExpression(expr) => self.resolve_binary(expr),
            Expressions::AssignmentExpression(expr) => self.resolve_assignment(expr),
            Expressions::RelationalExpression(expr) => Ok(self.resolve_relation(expr)?.into()),
            Expressions::EqualityExpression(expr) => Ok(self.resolve_equality(expr)?.into()),
            Expressions::UnaryExpression(expr) => Ok(self.resolve_unary(expr)?.into()),
            Expressions::LogicalExpression(expr) => Ok(self.resolve_logical(expr)?.into()),
            Expressions::CallExpressionn(expr) => Ok(self.resolve_call(expr)?.into()),
            Expressions::Literal(_) => Ok(expr),
        }
    }

    pub fn visit_block<'a>(
        &mut self,
        block: BlockStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        self.push_local_scope();
        let block = BlockStatement::new(
            self.resolve_statements(block.statements)?,
            block.begin_line,
            block.end_line,
        );
        self.pop_scope();
        Ok(block.into())
    }

    pub fn visit_expr<'a>(
        &mut self,
        expr: ExprStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        let expr = ExprStatement::new(self.resolve_expr(expr.expr)?);
        Ok(expr.into())
    }

    pub fn visit_print<'a>(
        &mut self,
        print: PrintStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        let print = PrintStatement::new(self.resolve_expr(print.expr)?);
        Ok(print.into())
    }
    pub fn visit_return<'a>(
        &mut self,
        print: ReturnStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        let print = ReturnStatement::new(print.expr.map(|e| self.resolve_expr(e)).transpose()?, print.line_number);
        Ok(print.into())
    }

    pub fn visit_declare<'a>(
        &mut self,
        mut declare: DeclareStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        match self.scope.get_local_scope_mut() {
            Some(v) => {
                v.declare_identifier(&mut declare.ident, self.stack_index )?;
                self.stack_index += 1;
            }
            None => {}
        };

        let declare = DeclareStatement::new(
            declare.ident,
            match declare.expr {
                Some(a) => Some(self.resolve_expr(a)?),
                None => None,
            },
        );

        Ok(declare.into())
    }

    pub fn visit_if<'a>(
        &mut self,
        statement: IfStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        let mut new_statements = vec![];
        for (expr, stat, l) in statement.statements {
            new_statements.push((
                expr.map(|expr| self.resolve_expr(expr)).transpose()?,
                self.resolve_statement(stat)?,
                l,
            ));
        }

        return Ok(IfStatement::new(new_statements).into());
    }

    pub fn visit_while<'a>(
        &mut self,
        statement: WhileStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        let expr = self.resolve_expr(statement.expression)?;
        let statement = self.resolve_statement(*statement.statement)?;

        return Ok(WhileStatement::new(expr, Box::new(statement)).into());
    }

    pub fn visit_for<'a>(
        &mut self,
        statement: ForStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        self.push_local_scope();

        let dec = statement
            .variable_declare
            .map(|e| self.resolve_statement(*e).map(Box::new))
            .transpose()?;
        let test = statement.test.map(|e| self.resolve_expr(e)).transpose()?;
        let inc = statement.inc.map(|e| self.resolve_expr(e)).transpose()?;

        let exec = self.resolve_statement(*statement.statement).map(Box::new)?;
        self.pop_scope();

        return Ok(ForStatement::new(
            dec,
            test,
            inc,
            exec,
            statement.begin_line,
            statement.end_line,
        )
        .into());
    }

    pub fn visit_function_declare<'a>(
        &mut self,
        mut statement: FunctionDeclareStatement<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        match self.scope.get_local_scope_mut() {
            Some(v) => {
                v.declare_identifier(&mut statement.ident, self.stack_index)?;
                self.stack_index += 1;
            }
            None => {}
        };
        self.push_local_scope();
        self.stack_index = 0;
        match self.scope.get_local_scope_mut() {
            Some(v) => {
                statement
                    .args
                    .iter_mut()
                    
                    .map(|mut arg| {
                        let result = v.declare_identifier(&mut arg, self.stack_index);
                        self.stack_index += 1;
                        result
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            }
            None => unreachable!(),
        };

        let statements = std::mem::replace(&mut statement.statements, vec![]);
        statement.statements = self.resolve_statements(statements)?;
        self.pop_scope();

        return Ok(statement.into());
    }

    pub fn resolve_statement<'a>(
        &mut self,
        statement: Statements<'a>,
    ) -> Result<Statements<'a>, ParserError> {
        match statement {
            Statements::DeclareStatement(declare_statement) => {
                self.visit_declare(declare_statement)
            }
            Statements::BlockStatement(block_statement) => self.visit_block(block_statement),
            Statements::ExprStatement(expr_statement) => self.visit_expr(expr_statement),
            Statements::PrintStatement(print_statement) => self.visit_print(print_statement),
            Statements::IfStatement(if_statement) => self.visit_if(if_statement),
            Statements::WhileStatement(while_statement) => self.visit_while(while_statement),
            Statements::ForStatement(for_statement) => self.visit_for(for_statement),
            Statements::FunctionDeclareStatement(function_declare_statement) => {
                self.visit_function_declare(function_declare_statement)
            }
            Statements::ReturnStatement(return_statement) => self.visit_return(return_statement),
        }
    }

    pub fn resolve_statements<'a>(
        &mut self,
        statements: Vec<Statements<'a>>,
    ) -> Result<Vec<Statements<'a>>, ParserError> {
        statements
            .into_iter()
            .map(|stmt| self.resolve_statement(stmt))
            .collect()
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
