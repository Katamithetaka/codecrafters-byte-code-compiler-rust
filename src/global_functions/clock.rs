use crate::expressions::Value;

pub fn execute(_: Vec<Value<String>>) -> Value<String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    Value::Number(since_epoch.as_secs_f64())
}

pub const NAME: &str = "clock";
pub const NUM_ARGUMENTS: u8 = 0;
