use crate::expressions::Value;

pub fn execute(_: Vec<Value>) -> Value {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    Value::number(since_epoch.as_secs_f64())
}

pub const NAME: &str = "clock";
pub const NUM_ARGUMENTS: Option<u8> = Some(0);
