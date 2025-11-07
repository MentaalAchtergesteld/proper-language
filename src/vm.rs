use std::{array, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{compiler::{Closure, FunctionPrototype, Instruction, Value}, properstd::native_print};

struct CallFrame {
    closure: Rc<Closure>,
    ip: usize,
    stack_base: usize
}

impl CallFrame {
    pub fn new(closure: Rc<Closure>, stack_base: usize) -> Self {
        Self {
            closure,
            ip: 0,
            stack_base
        }
    }
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new(main_proto: FunctionPrototype, globals: HashMap<String, Value>) -> Self {
        let main_closure = Rc::new(Closure {
            prototype: Rc::new(main_proto),
            upvalues: Vec::new()
        });

        let main_frame = CallFrame::new(main_closure, 0);
        VM {
            frames: vec![main_frame],
            stack: Vec::new(),
            globals
        }
    }

    fn print_stack(&self) {
        for (i, val) in self.stack.iter().enumerate() {
            println!("{i}: {val}");
        }
    }

    pub fn run(&mut self) {
        loop {
            let frame = self.frames.last_mut().unwrap();
            let instruction = &frame.closure.prototype.bytecode[frame.ip];
            frame.ip += 1;

            match instruction {
                Instruction::PushConst(index) => {
                    let constant = &frame.closure.prototype.constant_pool[*index];
                    self.stack.push(constant.clone());
                },

                Instruction::StoreLocal(index) => {
                    let value = self.stack.pop().unwrap();

                    self.stack[frame.stack_base + index] = value;
                },
                Instruction::LoadLocal(index) => {
                    let value = self.stack[frame.stack_base + index].clone();
                    self.stack.push(value);
                },

                Instruction::StoreUpvalue(index) => {
                    let value = self.stack.pop().unwrap();
                    let upvalue = &frame.closure.upvalues[*index];
                    *upvalue.borrow_mut() = value;
                },
                Instruction::LoadUpvalue(index) => {
                    let upvalue = &frame.closure.upvalues[*index];
                    let value = upvalue.borrow().clone();
                    self.stack.push(value);
                },

                Instruction::StoreIndex => {
                    let value = self.stack.pop().unwrap();
                    let index_val = self.stack.pop().unwrap();
                    let array_val = self.stack.pop().unwrap();

                    let array = array_val.as_array();
                    let index = index_val.as_integer() as usize;

                    array.borrow_mut()[index] = value.clone();
                    self.stack.push(value);
                },
                Instruction::LoadIndex => {
                    let index_val = self.stack.pop().unwrap();
                    let array_val = self.stack.pop().unwrap();

                    let array = array_val.as_array();
                    let index = index_val.as_integer() as usize;

                    let value = array.borrow()[index].clone();
                    self.stack.push(value);
                },

                Instruction::StoreField(index) => {
                    let value = self.stack.pop().unwrap();
                    let struct_val = self.stack.pop().unwrap();

                    let field_name = frame.closure.prototype.constant_pool[*index].as_string();
                    let struct_ = struct_val.as_struct();

                    struct_.borrow_mut().insert(field_name, value.clone());
                    self.stack.push(value);
                },
                Instruction::LoadField(index) => { 
                    let struct_val = self.stack.pop().unwrap();

                    let field_name = frame.closure.prototype.constant_pool[*index].as_string();
                    let struct_ = struct_val.as_struct();

                    let value = struct_.borrow()[&field_name].clone();
                    self.stack.push(value);
                },

                Instruction::StoreGlobal(index) => {
                    let value = self.stack.pop().unwrap();
                    let field_name = frame.closure.prototype.constant_pool[*index].as_string();
                    self.globals.insert(field_name, value);
                },
                Instruction::LoadGlobal(index) => {
                    let field_name = frame.closure.prototype.constant_pool[*index].as_string();
                    let value = self.globals.get(&field_name).unwrap().clone();
                    self.stack.push(value);
                },

                Instruction::Or => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Bool(left), Value::Bool(right)) => self.stack.push(Value::Bool(left || right)),
                        _ => todo!("todo: can't OR non bool values yet"),
                    }
                },
                Instruction::And => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Bool(left), Value::Bool(right)) => self.stack.push(Value::Bool(left && right)),
                        _ => todo!("todo: can't AND non bool values yet"),
                    }
                },
                Instruction::Equal => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Bool(left == right)),
                        (Value::Bool(left), Value::Bool(right)) => self.stack.push(Value::Bool(left == right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Bool(left == right)),
                        (Value::String(left), Value::String(right)) => self.stack.push(Value::Bool(left == right)),
                        _ => todo!("todo: can't EQUAL non integer or bool values yet"),
                    }
                },
                Instruction::NotEqual => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Bool(left != right)),
                        (Value::Bool(left), Value::Bool(right)) => self.stack.push(Value::Bool(left != right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Bool(left != right)),
                        (Value::String(left), Value::String(right)) => self.stack.push(Value::Bool(left != right)),
                        _ => todo!("todo: can't EQUAL non integer or bool values yet"),
                    }
                },
                Instruction::GreaterThan => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Bool(left > right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Bool(left > right)),
                        _ => todo!("todo: can't GREATER THAN non integer or float values")
                    }
                },
                Instruction::GreaterThanEqual => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Bool(left >= right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Bool(left >= right)),
                        _ => todo!("todo: can't GREATER THAN EQUAL non integer or float values")
                    }
                },
                Instruction::LessThan => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Bool(left < right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Bool(left < right)),
                        _ => todo!("todo: can't LESS THAN non integer or float values")
                    }
                },
                Instruction::LessThanEqual => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Bool(left <= right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Bool(left <= right)),
                        _ => todo!("todo: can't LESS THAN EQUAL non integer or float values")
                    }
                },

                Instruction::Add => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Integer(left + right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Float(left + right)),
                        (Value::Float(left), Value::Integer(right)) => self.stack.push(Value::Float(left + right as f32)),
                        (Value::Integer(left), Value::Float(right)) => self.stack.push(Value::Float(left as f32 + right)),
                        _ => todo!("todo: can't ADD non integer or float values")
                    }
                },
                Instruction::Subtract => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Integer(left - right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Float(left - right)),
                        (Value::Float(left), Value::Integer(right)) => self.stack.push(Value::Float(left - right as f32)),
                        (Value::Integer(left), Value::Float(right)) => self.stack.push(Value::Float(left as f32 - right)),
                        _ => todo!("todo: can't SUBTRACT non integer or float values")
                    }
                },
                Instruction::Multiply => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Integer(left * right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Float(left * right)),
                        (Value::Float(left), Value::Integer(right)) => self.stack.push(Value::Float(left * right as f32)),
                        (Value::Integer(left), Value::Float(right)) => self.stack.push(Value::Float(left as f32 * right)),
                        _ => todo!("todo: can't MULTIPLY non integer or float values")
                    }
                },
                Instruction::Divide => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Integer(left / right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Float(left / right)),
                        (Value::Float(left), Value::Integer(right)) => self.stack.push(Value::Float(left / right as f32)),
                        (Value::Integer(left), Value::Float(right)) => self.stack.push(Value::Float(left as f32 / right)),
                        _ => todo!("todo: can't DIVIDE non integer or float values")
                    }
                },
                Instruction::Modulo => { 
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Integer(left), Value::Integer(right)) => self.stack.push(Value::Integer(left % right)),
                        (Value::Float(left), Value::Float(right)) => self.stack.push(Value::Float(left % right)),
                        (Value::Float(left), Value::Integer(right)) => self.stack.push(Value::Float(left % right as f32)),
                        (Value::Integer(left), Value::Float(right)) => self.stack.push(Value::Float(left as f32 % right)),
                        _ => todo!("todo: can't MODULO non integer or float values")
                    }
                },

                Instruction::Not => {
                    let right = self.stack.pop().unwrap();
                    match right {
                        Value::Bool(right) => self.stack.push(Value::Bool(!right)),
                        _ => todo!("todo: can't NOT non bool value")
                    }
                },
                Instruction::Negate => { 
                    let right = self.stack.pop().unwrap();
                    match right {
                        Value::Integer(right) => self.stack.push(Value::Integer(-right)),
                        Value::Float(right) => self.stack.push(Value::Float(-right)),
                        _ => todo!("todo: can't NEGATE non integer or float value")
                    }
                },

                Instruction::Call(arg_count) => {
                    let callee = &self.stack.pop().unwrap();

                    match callee {
                        Value::Closure(closure) => {
                            let new_frame = CallFrame::new(closure.clone(), self.stack.len()-1);
                            self.frames.push(new_frame);
                        },
                        Value::NativeFn(native_fn) => {
                            let start_index = self.stack.len() - arg_count;
                            let args = self.stack.drain(start_index..).collect::<Vec<Value>>();
                            let result = (native_fn.function)(&args);
                            self.stack.push(result);
                        },
                        _ => panic!("ERROR: can't call non-function"),
                    }
                },
                Instruction::CreateArray(element_count) => {
                    let start_index = self.stack.len() - element_count;
                    let elements = self.stack.drain(start_index..).collect::<Vec<Value>>();
                    let array = Rc::new(RefCell::new(elements));
                    self.stack.push(Value::Array(array));

                },
                Instruction::CreateStruct(field_count) => {
                    let mut map = HashMap::new();
                    for _ in 0..*field_count {
                        let value = self.stack.pop().unwrap();
                        let key_val = self.stack.pop().unwrap();
                        let key_str = key_val.as_string();
                        map.insert(key_str, value);
                    }

                    let val = Rc::new(RefCell::new(map));
                    self.stack.push(Value::Struct(val));

                },
                Instruction::CreateClosure(upvalue_count) => {
                    let mut upvalues = Vec::with_capacity(*upvalue_count);
                    for _ in 0..*upvalue_count {
                        let upvalue = Rc::new(RefCell::new(self.stack.pop().unwrap()));
                        upvalues.push(upvalue);
                    }
                    upvalues.reverse();

                    let proto_val = self.stack.pop().unwrap();
                    let proto = match proto_val {
                        Value::Function(proto) => proto.clone(),
                        _ => panic!("ERORR: can't creater closure from non function value")
                    };
                    let closure = Rc::new(Closure { prototype: proto, upvalues });
                    self.stack.push(Value::Closure(closure));
                },

                Instruction::JumpIfZero(address) => {
                    let value = self.stack.pop().unwrap();

                    let is_truthy = match value {
                        Value::Bool(b) => b,
                        _ => true,
                    };

                    if !is_truthy {
                        frame.ip = *address;
                    }
                },
                Instruction::Jump(address) => {
                    frame.ip = *address;
                },

                Instruction::Pop(count) => {
                    for _ in 0..*count { self.stack.pop(); }
                },
                Instruction::Duplicate(count) => {
                    let value = self.stack.last().unwrap().clone();
                    for _ in 0..*count { self.stack.push(value.clone()); }
                },
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap_or(Value::Bool(true));
                    let finished_frame = self.frames.pop().unwrap();

                    self.stack.truncate(finished_frame.stack_base);
                    self.stack.push(return_value);
                },
                Instruction::Halt => { break },
            }
        }
    }
}
