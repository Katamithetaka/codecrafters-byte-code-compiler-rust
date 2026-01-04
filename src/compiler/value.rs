pub type Value = crate::expressions::Value;
pub type ValueArray = Vec<Value>;

pub fn print_value(value: &Value) {
    eprintln!("'{value}'");
}
