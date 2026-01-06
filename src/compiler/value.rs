use std::fmt::Display;

use crate::expressions::Value;

pub fn print_value<S: Display>(value: &Value<S>) {
    eprintln!("'{value}'");
}
