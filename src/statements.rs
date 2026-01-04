use std::fmt::Debug;

use crate::compiler::CodeGenerator;

pub mod print_statement;

pub trait Statement: Debug + CodeGenerator{}

pub mod prelude {}
