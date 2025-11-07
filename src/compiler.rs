use std::{cell::RefCell, collections::{HashMap, HashSet}, fmt, io::{self, Write}, rc::Rc};

use crate::{astbuilder::{BinaryOperator, Expression, LiteralValue, Statement, UnaryOperator}, properstd::NativeFunction};

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushConst(usize),

    StoreLocal(usize),
    LoadLocal(usize),

    StoreUpvalue(usize),
    LoadUpvalue(usize),

    StoreIndex,
    LoadIndex,

    StoreField(usize),
    LoadField(usize),

    StoreGlobal(usize),
    LoadGlobal(usize),
    
    Or,
    And,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    Not,
    Negate,

    Call(usize),
    CreateArray(usize),
    CreateStruct(usize),
    CreateClosure(usize),

    JumpIfZero(usize),
    Jump(usize),

    Pop(usize),
    Duplicate(usize),
    Return,
    Halt
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::PushConst(idx) => write!(f, "PushConst {idx}"),

            Instruction::StoreLocal(idx) => write!(f, "StoreLocal {idx}"),
            Instruction::LoadLocal(idx) => write!(f, "LoadLocal {idx}"),

            Instruction::StoreUpvalue(idx) => write!(f, "StoreUpvalue {idx}"),
            Instruction::LoadUpvalue(idx) => write!(f, "LoadUpvalue {idx}"),

            Instruction::StoreIndex => write!(f, "StoreIndex"),
            Instruction::LoadIndex => write!(f, "LoadIndex"),

            Instruction::StoreField(idx) => write!(f, "StoreField {idx}"),
            Instruction::LoadField(idx) => write!(f, "StoreField {idx}"),

            Instruction::StoreGlobal(idx) => write!(f, "StoreGlobal {idx}"),
            Instruction::LoadGlobal(idx) => write!(f, "LoadGlobal {idx}"),

            Instruction::Or => write!(f, "Or"),
            Instruction::And=> write!(f, "And"),
            Instruction::Equal => write!(f, "Equal"),
            Instruction::NotEqual => write!(f, "NotEqual"),
            Instruction::GreaterThan => write!(f, "GreaterThan"),
            Instruction::GreaterThanEqual => write!(f, "GreaterThanEqual"),
            Instruction::LessThan => write!(f, "LessThan"),
            Instruction::LessThanEqual => write!(f, "LessThanEqual"),

            Instruction::Add => write!(f, "Add"),
            Instruction::Subtract=> write!(f, "Subtract"),
            Instruction::Multiply=> write!(f, "Multiply"),
            Instruction::Divide => write!(f, "Divide"),
            Instruction::Modulo => write!(f, "Modulo"),

            Instruction::Not => write!(f, "Not"),
            Instruction::Negate => write!(f, "Negate"),

            Instruction::Call(count) => write!(f, "Call {count}"),
            Instruction::CreateArray(count) => write!(f, "CreateArray {count}"),
            Instruction::CreateStruct(count) => write!(f, "CreateStruct {count}"),
            Instruction::CreateClosure(idx) => write!(f, "CreateClosure {idx}"),

            Instruction::JumpIfZero(addr) => write!(f, "JumpIfZero {addr}"),
            Instruction::Jump(addr) => write!(f, "Jump {addr}"),

            Instruction::Duplicate(count) => write!(f, "Duplicate {count}"),
            Instruction::Pop(count) => write!(f, "Pop {count}"),
            Instruction::Return => write!(f, "Return"),
            Instruction::Halt => write!(f, "Halt")
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Function(Rc<FunctionPrototype>),
    Closure(Rc<Closure>),
    Array(Rc<RefCell<Vec<Value>>>),
    Struct(Rc<RefCell<HashMap<String, Value>>>),
    NativeFn(NativeFunction),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "Integer {i}"),
            Value::Float(fl) => write!(f, "Float {fl}"),
            Value::String(s) => write!(f, "String {s}"),
            Value::Bool(b) => write!(f, "Bool {b}"),
            Value::Function(p) => write!(f, "Function <{}> (arity {})", p.name, p.arity),
            Value::Closure(c) => write!(f, "Closure <{}> (arity {})", c.prototype.name, c.prototype.arity),
            Value::Array(a) => write!(f, "Array (length {})", a.borrow().len()),
            Value::Struct(s) => write!(f, "Struct (field count {})", s.borrow().len()),
            Value::NativeFn(nf) => write!(f, "NativeFn {:?}", nf),
        }
    }
}

impl Value {
    pub fn as_integer(&self) -> i32 {
        match self {
            Value::Integer(i) => *i,
            _ => panic!("ERROR: expected integer"),
        }
    }

    pub fn as_string(&self) -> String {
         match self {
            Value::String(s) => s.clone(),
            _ => panic!("ERROR: expected string"),
        }
    }
    //
    // pub fn as_upvalue_object(&self) -> Rc<RefCell<Value>> {
    //     match self {
    //         Value::Upvalue(rc) => rc.clone(),
    //         _ => panic!("ERROR: expected upvalue"),
    //     }
    // }

    pub fn as_array(&self) -> Rc<RefCell<Vec<Value>>> {
         match self {
            Value::Array(arr) => arr.clone(),
            _ => panic!("ERROR: expected array"),
        }
    }

    pub fn as_struct(&self) -> Rc<RefCell<HashMap<String, Value>>> {
         match self {
            Value::Struct(s) => s.clone(),
            _ => panic!("ERROR: expected struct"),
        }
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}


#[derive(Clone, PartialEq)]
pub struct FunctionPrototype {
    pub name: String,
    pub arity: usize,
    pub upvalue_count: usize,

    pub bytecode: Vec<Instruction>,
    pub constant_pool: Vec<Value>,
}

impl FunctionPrototype {
    pub fn new(name: String, arity: usize) -> Self {
        Self {
            name,
            arity,
            upvalue_count: 0,
            bytecode: Vec::new(),
            constant_pool: Vec::new()
        }
    }
    pub fn new_empty() -> Self {
        Self {
            name: "empty".to_string(),
            arity: 0,
            upvalue_count: 0,
            bytecode: Vec::new(),
            constant_pool: Vec::new()
        }
    }
}

#[derive(Clone)]
pub struct Closure {
    pub prototype: Rc<FunctionPrototype>,
    pub upvalues: Vec<Rc<RefCell<Value>>>
}

struct Local {
    name: String,
    depth: usize,
}


#[derive(Debug)]
pub enum CompilerError {
    UndefinedVariable(String),
    VariableAlreadyExists(String),
    InvalidAssignmentTarget,
    TopLevelReturn,
}

struct FunctionContext {
    prototype: FunctionPrototype,
    locals: Vec<Local>,
    upvalues: Vec<String>,
    current_depth: usize,
}

impl FunctionContext {
    pub fn new(prototype: FunctionPrototype) -> Self {
        Self {
            prototype,
            locals: Vec::new(),
            upvalues: Vec::new(),
            current_depth: 0,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            prototype: FunctionPrototype::new_empty(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            current_depth: 0,
        }
    }

    pub fn push_scope(&mut self) {
        self.current_depth+=1;
    }

    pub fn pop_scope(&mut self) {
        self.current_depth-=1;

        while let Some(local) = self.locals.last() {
            if local.depth <= self.current_depth { break; }
            self.locals.pop();
        }
    }

    pub fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name { return Some(i) }
        }
        None
    }

    pub fn create_local(&mut self, name: &str) -> Result<usize, CompilerError> {
        if self.resolve_local(name).is_some() { return Err(CompilerError::VariableAlreadyExists(name.to_string())) }
        let local = Local {
            name: name.to_string(),
            depth: self.current_depth
        };
        self.locals.push(local);
        Ok(self.locals.len()-1)
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        let pool = &self.prototype.constant_pool;
        if let Some(index) = pool.iter().position(|v| *v == value) {
            return index;
        }

        let pool_mut = &mut self.prototype.constant_pool;
        pool_mut.push(value);
        pool_mut.len()-1
    }
}

pub struct Compiler {
    functions: Vec<FunctionContext>,
    globals: HashSet<String>,
}

impl Compiler {
    pub fn new(globals: HashSet<String>) -> Self {
        Self { functions: vec![FunctionContext::new_empty()], globals }
    }

    fn current_fn(&self) -> &FunctionContext {
        self.functions.last().unwrap()
    } 

    fn current_fn_mut(&mut self) -> &mut FunctionContext {
        self.functions.last_mut().unwrap()
    }

    fn push_context(&mut self, name: String, arity: usize) {
        let prototype = FunctionPrototype::new(name, arity);
        let context = FunctionContext::new(prototype);
        self.functions.push(context);
    }

    fn pop_context(&mut self) -> FunctionContext {
        self.functions.pop().unwrap()
    }

    fn emit(&mut self, instr: Instruction) -> usize {
        self.current_fn_mut().prototype.bytecode.push(instr);
        self.current_fn().prototype.bytecode.len()-1
    }

    fn patch_jump(&mut self, jump_index: usize) {
        let target = self.current_fn().prototype.bytecode.len();
        let instr = &mut self.current_fn_mut().prototype.bytecode[jump_index];
        match instr {
            Instruction::JumpIfZero(addr) => *addr = target,
            Instruction::Jump(addr) => *addr = target,
            _ => panic!("ERROR: can't patch a non-jump instruction")
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        self.current_fn().resolve_local(name)
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        if let Some(index) = self.current_fn().upvalues.iter().position(|u| u == name) {
            return Some(index);
        }

        let mut found = false;
        for context in self.functions.iter().rev().skip(1) {
            if context.resolve_local(name).is_some() { found = true; break; }
        }

        if found {
            self.current_fn_mut().upvalues.push(name.to_string());
            Some(self.current_fn().upvalues.len()-1)
        } else {
            None
        }
    }

    fn resolve_and_emit_load(&mut self, name: &str) -> Result<(), CompilerError> {
        if let Some(index) = self.resolve_local(name) {
            self.emit(Instruction::LoadLocal(index));
            return Ok(());
        }

        if let Some(index) = self.resolve_upvalue(name) {
            self.emit(Instruction::LoadUpvalue(index));
            return Ok(())
        }

        if self.globals.contains(name) {
            let name_index = self.current_fn_mut().add_constant(Value::String(name.to_string()));
            self.emit(Instruction::LoadGlobal(name_index));
            return Ok(())
        }

        Err(CompilerError::UndefinedVariable(name.to_string()))
    }

    fn compile_expression(&mut self, expression: Expression) -> Result<(), CompilerError> {
        match expression {
            Expression::BinaryExpression { left, right, operator } => {
                self.compile_expression(*left)?;
                self.compile_expression(*right)?;
                let op_instr = match operator {
                    BinaryOperator::Or => Instruction::Or,
                    BinaryOperator::And => Instruction::And,
                    BinaryOperator::Equal => Instruction::Equal,
                    BinaryOperator::NotEqual => Instruction::NotEqual,
                    BinaryOperator::GreaterThan => Instruction::GreaterThan,
                    BinaryOperator::GreaterThanEqual => Instruction::GreaterThanEqual,
                    BinaryOperator::LessThan => Instruction::LessThan,
                    BinaryOperator::LessThanEqual => Instruction::LessThanEqual,
                    BinaryOperator::Add => Instruction::Add,
                    BinaryOperator::Subtract => Instruction::Subtract,
                    BinaryOperator::Multiply => Instruction::Multiply,
                    BinaryOperator::Divide => Instruction::Divide,
                    BinaryOperator::Modulo => Instruction::Modulo,
                };
                self.emit(op_instr);
                Ok(())
            },
            Expression::UnaryExpression { operator, right } => {
                self.compile_expression(*right)?;
                let op_instr = match operator {
                    UnaryOperator::Not => Instruction::Not,
                    UnaryOperator::Negate => Instruction::Negate
                };
                self.emit(op_instr);
                Ok(())
            },
            Expression::Literal(value) => {
                let value = match value {
                    LiteralValue::Integer(v) => Value::Integer(v),
                    LiteralValue::Float(v) => Value::Float(v),
                    LiteralValue::String(v) => Value::String(v),
                    LiteralValue::Bool(v) => Value::Bool(v)
                };

                let index = self.current_fn_mut().add_constant(value);
                self.emit(Instruction::PushConst(index));
                Ok(())
            },
            Expression::Variable(name) => {
                self.resolve_and_emit_load(&name)
            },
            Expression::Assign { variable, value } => {
                match *variable {
                    Expression::Variable(name) => {
                        self.compile_expression(*value)?;
                        self.emit(Instruction::Duplicate(1));

                        if let Some(index) = self.resolve_local(&name) {
                            self.emit(Instruction::StoreLocal(index));
                            return Ok(());
                        }

                        if let Some(index) = self.resolve_upvalue(&name) {
                            self.emit(Instruction::StoreUpvalue(index));
                            return Ok(())
                        }

                        Err(CompilerError::UndefinedVariable(name.to_string()))
                    },
                    Expression::Index { callee, index } => {
                        self.compile_expression(*callee)?;
                        self.compile_expression(*index)?;
                        self.compile_expression(*value)?;
                        self.emit(Instruction::Duplicate(1));
                        self.emit(Instruction::StoreIndex);
                        Ok(())
                    },
                    Expression::Field { callee, name } => {
                        self.compile_expression(*callee)?;
                        self.compile_expression(*value)?;
                        self.emit(Instruction::Duplicate(1));
                        let name_index = self.current_fn_mut().add_constant(Value::String(name));
                        self.emit(Instruction::StoreField(name_index));
                        Ok(())
                    },
                    _ => Err(CompilerError::InvalidAssignmentTarget)
                }
            },
            Expression::Call { callee, args } => {
                let arg_count = args.len();

                for arg in args {
                    self.compile_expression(arg)?;
                }

                self.compile_expression(*callee)?;

                self.emit(Instruction::Call(arg_count));
                Ok(())
            },
            Expression::Index { callee, index } => {
                self.compile_expression(*callee)?;
                self.compile_expression(*index)?;
                self.emit(Instruction::LoadIndex);
                Ok(())
            },
            Expression::Field { callee, name } => {
                self.compile_expression(*callee)?;
                let name_index = self.current_fn_mut().add_constant(Value::String(name));
                self.emit(Instruction::LoadField(name_index));
                Ok(())
            },
            Expression::Array(elements) => {
                let elements_count = elements.len();
                for element in elements {
                    self.compile_expression(element)?;
                }
                self.emit(Instruction::CreateArray(elements_count));
                Ok(())
            },
            Expression::Struct(fields) => {
                let field_count = fields.len();

                for (name, value) in fields {
                    let key_index = self.current_fn_mut().add_constant(Value::String(name.clone()));
                    self.emit(Instruction::PushConst(key_index));
                    self.compile_expression(value)?;
                }

                self.emit(Instruction::CreateStruct(field_count));
                Ok(())
            }
        }
    }

    fn compile_statement(&mut self, statement: Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::If { condition, block, else_branch } => {
                self.compile_expression(condition)?;
                let else_jump = self.emit(Instruction::JumpIfZero(0));

                self.compile_block(block)?;
                let end_jump = self.emit(Instruction::Jump(0));

                self.patch_jump(else_jump);

                if let Some(else_block) = else_branch {
                    self.compile_block(else_block)?;
                }
                self.patch_jump(end_jump);

                Ok(())
            },
            Statement::While { condition, block } => {
                let start_jump = self.current_fn().prototype.bytecode.len();

                self.compile_expression(condition)?;
                let end_jump = self.emit(Instruction::JumpIfZero(0));

                self.compile_block(block)?;
                self.emit(Instruction::Jump(start_jump));

                self.patch_jump(end_jump);

                Ok(())
            },
            Statement::FnDefinition { name, params, block } => {
                self.push_context(name.clone(), params.len());
                for param in params {
                    self.current_fn_mut().create_local(&param)?;
                }

                self.compile_block(block)?;

                let context = self.current_fn_mut();
                match context.prototype.bytecode.last() {
                    Some(Instruction::Return) => {},
                    _ => {
                        let slot = context.add_constant(Value::Bool(true));
                        self.emit(Instruction::PushConst(slot));
                        self.emit(Instruction::Return);
                    }
                }

                let finished_context = self.pop_context();
                let upvalue_count = finished_context.upvalues.len();
                let upvalues = finished_context.upvalues;

                let proto_index = self.current_fn_mut().add_constant(Value::Function(Rc::new(finished_context.prototype)));
                self.emit(Instruction::PushConst(proto_index));
                
                for upvalue_name in upvalues {
                    self.resolve_and_emit_load(&upvalue_name)?;
                }

                self.emit(Instruction::CreateClosure(upvalue_count));

                self.current_fn_mut().create_local(&name)?;

                Ok(())
            },
            Statement::Let { name, value } => {
                self.compile_expression(value)?;
                self.current_fn_mut().create_local(&name)?;
                Ok(())
            },
            Statement::Return { value } => {
                if self.functions.len() <= 1 { return Err(CompilerError::TopLevelReturn) };
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                } else {
                    let bool_index = self.current_fn_mut().add_constant(Value::Bool(true));
                    self.emit(Instruction::PushConst(bool_index));
                }
                self.emit(Instruction::Return);
                Ok(())
            },
            Statement::Continue => { todo!() },
            Statement::Break => { todo!() },
            Statement::Expression { value } => {
                self.compile_expression(value)?;
                self.emit(Instruction::Pop(1));
                Ok(())
            }
        }
    }

    fn compile_block(&mut self, block: Vec<Statement>) -> Result<(), CompilerError> {
        self.current_fn_mut().push_scope();
        for stmt in block {
            self.compile_statement(stmt)?;
        }

        self.current_fn_mut().pop_scope();
        Ok(())
    }

    pub fn compile(&mut self, code: Vec<Statement>) -> Result<FunctionPrototype, CompilerError> {
        self.functions = vec![FunctionContext::new_empty()];
        self.compile_block(code)?;
        self.emit(Instruction::Halt);
        Ok(self.pop_context().prototype)
    }
}

fn disassemble_recursive(proto: &FunctionPrototype, writer: &mut impl Write, depth:usize) -> io::Result<()> {
    let indent = " ".repeat(depth);

    writeln!(writer, "{indent}[FUNCTION <{}> (arity {})]", proto.name, proto.arity)?;
    writeln!(writer, "{indent} --- Constants ---")?;
    for (i, value) in proto.constant_pool.iter().enumerate() {
        write!(writer, "{indent} {i:04} ")?;

        match value {
            Value::Function(nested_proto) => {
                writeln!(writer)?;
                disassemble_recursive(nested_proto, writer, depth+1)?;
            },
            _ => writeln!(writer, "{value}")?,
        }
    }

    writeln!(writer, "{} --- Code ---", indent)?;
    for (i, instruction) in proto.bytecode.iter().enumerate() {
        writeln!(writer, "{indent} {i:04} {instruction}")?;
    }
    writeln!(writer, "{indent}[END FUNCTION <{}>]", proto.name)?;
    Ok(())
}

pub fn disassemble_program(main_function: &FunctionPrototype, writer: &mut impl Write) -> io::Result<()> {
    disassemble_recursive(main_function, writer, 0)
} 
