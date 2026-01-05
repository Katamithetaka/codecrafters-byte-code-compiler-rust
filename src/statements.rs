use std::fmt::Debug;

use crate::compiler::CodeGenerator;

pub mod print_statement;
pub mod expression_statement;
pub mod declare_statement;

pub trait Statement: Debug + CodeGenerator{}

pub mod prelude {}
