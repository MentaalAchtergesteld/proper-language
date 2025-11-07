use std::fmt::Debug;

use crate::compiler::Value;

pub type NativeFn = fn(args: &[Value]) -> Value;

#[derive(Clone)]
pub struct NativeFunction {
    pub name: &'static str,
    pub function: NativeFn,
}

impl NativeFunction {
    pub fn new(name: &'static str, function: NativeFn) -> Self {
        NativeFunction { name, function }
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.function as *const () == other.function as *const ()
    }
}

pub fn native_print(args: &[Value]) -> Value {
    for (i, arg) in args.iter().enumerate() {
        print!("{}", arg);
        if i < args.len() - 1 {
            print!(" ");
        }
    }
    println!();
    Value::Bool(true)
}

pub fn native_clock(_args: &[Value]) -> Value {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();
    Value::Float(secs)
}
