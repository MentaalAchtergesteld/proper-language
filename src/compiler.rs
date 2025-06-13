use std::{cell::RefCell, collections::HashMap, rc::Rc, usize};

use crate::parser::{BinaryOperator, ElseBranch, Expression, Statement};

pub enum CompilerError {
    UndefinedVariable(String),
    WrongArgCount { function: String, expected: usize, got: usize },
    InvalidAssignment,
    TypeMismatch,
    InvalidUpvalue(String),
    UnexpectedStatement {
        expected: String,
        found: String
    },
    NoScope,
    NoFunctionScope,
    BreakOrContinueOutsideLoop,
}

type SharedValue = Rc<RefCell<Value>>;

pub enum Value {
    Undefined,
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(String),
    Closure {
        function_index: usize,
        upvalues: Vec<SharedValue>,
    }
}

pub enum Instruction {
    NoOp,
    PushValue(Value),

    Load(usize),
    Store(usize),
    LoadUpvalue(usize),
    StoreUpvalue(usize),

    Add,
    Sub,
    Mul,
    Div,

    Equal,
    NotEqual,
    Lesser,
    LesserEqual,
    Greater,
    GreaterEqual,

    And,
    Or,
    Not,

    CreateClosure { function_index: usize, upvalue_count: usize },
    Call,
    Return,

    Jump(usize),
    JumpIfFalse(usize),

    ArraySet,
    FieldSet
}

enum ResolvedVar {
    Local(usize),
    Upvalue(usize)
}

struct CompiledFunction { 
    pub instructions: Vec<Instruction>,
    pub upvalues: Vec<String>,
    pub parameter_count: usize,
    pub index: usize,
}

enum Scope {
    Function {
        locals: HashMap<String, usize>,
        upvalues: HashMap<String, usize>,
        stack_pointer: usize
    },
    Block {
        locals: HashMap<String, usize>,
        stack_pointer: usize,
    }
}

impl Scope {
    fn get_stack_pointer(&self) -> usize {
        match self {
            Self::Function { stack_pointer, .. } => *stack_pointer,
            Self::Block { stack_pointer, .. } => *stack_pointer
        }
    }

    fn push_stack(&mut self) {
        match self {
            Self::Function { stack_pointer, .. } => *stack_pointer += 1,
            Self::Block { stack_pointer, .. } => *stack_pointer += 1,
        }
    }

    fn pop_stack(&mut self) {
        match self {
            Self::Function { stack_pointer, .. } => *stack_pointer -= 1,
            Self::Block { stack_pointer, .. } => *stack_pointer -= 1,
        }
    }

    fn get_locals(&self) -> &HashMap<String, usize> {
        match self {
            Self::Function { locals, .. } => locals,
            Self::Block { locals, .. } => locals
        }
    }

    fn get_local(&self, name: &str) -> Option<usize> {
        self.get_locals().get(name).copied()
    }

    fn add_local(&mut self, name: String) -> Option<usize> {
        let sp = self.get_stack_pointer();
        let locals = match self {
            Self::Function { locals, .. } => locals,
            Self::Block { locals, .. } => locals,
        };
        locals.insert(name, sp);
        self.push_stack();
        Some(sp)
    }

    fn get_upvalues(&self) -> Option<&HashMap<String, usize>> {
        match self {
            Self::Function { upvalues, .. } => Some(upvalues),
            _ => None
        }
    }

    fn get_upvalue(&self, name: &str) -> Option<usize> {
        self.get_upvalues()?.get(name).copied()
    }

    fn add_upvalue(&mut self, name: String) -> Option<usize> {
        match self {
            Self::Function { upvalues, .. } => {
                let index = upvalues.len();
                upvalues.insert(name, index);
                Some(index)
            },
            _ => None
        }
    }
}

struct LoopContext {
    start: usize,
    breaks: Vec<usize>,
}

pub struct Compiler {
    scopes: Vec<Scope>,
    function_prototypes: HashMap<String, CompiledFunction>,
    loop_context: Vec<LoopContext>
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            scopes: Vec::new(),
            function_prototypes: HashMap::new(),
            loop_context: Vec::new() 
        }
    }

    fn push_function_scope(&mut self) {
        let scope = Scope::Function { locals: HashMap::new(), upvalues: HashMap::new(), stack_pointer: 0 };
        self.scopes.push(scope);
    }

    fn push_block_scope(&mut self) {
        let (locals, stack_pointer) = if let Some(scope) = self.scopes.last() {
            (scope.get_locals().clone(), scope.get_stack_pointer())
        } else {
            (HashMap::new(), 0)
        };

        let scope = Scope::Block { locals, stack_pointer };
        self.scopes.push(scope);
    }

    fn get_current_scope(&mut self) -> Result<&mut Scope, CompilerError> {
        self.scopes.last_mut().ok_or(CompilerError::NoScope)
    }

    fn get_loop_context(&mut self) -> Result<&mut LoopContext, CompilerError> {
        self.loop_context.last_mut().ok_or(CompilerError::BreakOrContinueOutsideLoop)
    }

    fn get_current_function_scope(&mut self) -> Result<&mut Scope, CompilerError> {
        self.scopes
            .iter_mut()
            .rev()
            .find(|s| matches!(s, Scope::Function { .. }))
            .ok_or(CompilerError::NoFunctionScope)
    }

    fn compile_expression(&mut self, expression: Expression, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        Ok(())
    }

    fn compile_if(&mut self, condition: Expression, then_branch: Vec<Statement>, else_branch: Option<ElseBranch>, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        self.compile_expression(condition, instructions)?;
        
        let jump_to_else_index = instructions.len();
        instructions.push(Instruction::NoOp);

        self.compile_block(then_branch, instructions)?;
        
        if else_branch.is_some() {
            instructions.push(Instruction::NoOp);
        };

        let else_start = instructions.len();
        instructions[jump_to_else_index] = Instruction::JumpIfFalse(else_start);

        if let Some(else_branch) = else_branch {
            match else_branch {
                ElseBranch::If(statement) => self.compile_statement(*statement, instructions)?,
                ElseBranch::Else(block) => self.compile_block(block, instructions)?,
            }

            let after_else = instructions.len();
            instructions[else_start-1] = Instruction::Jump(after_else);
        }

        Ok(())
    }

    fn compile_while(&mut self, condition: Expression, block: Vec<Statement>, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        let while_start = instructions.len();
        self.loop_context.push(LoopContext { start: while_start, breaks: Vec::new() });
        
        self.compile_expression(condition, instructions)?;
        
        let jump_over_index = instructions.len();
        instructions.push(Instruction::NoOp);

        self.compile_block(block, instructions)?; 

        instructions.push(Instruction::Jump(while_start));
        
        let after_loop = instructions.len();
        instructions[jump_over_index] = Instruction::JumpIfFalse(after_loop);

        for index in &self.get_loop_context()?.breaks {
            instructions[*index] = Instruction::Jump(after_loop);
        } 

        self.loop_context.pop();

        Ok(())
    }

    fn compile_break(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        let context = self.get_loop_context()?;

        let index = instructions.len();
        instructions.push(Instruction::NoOp);
        context.breaks.push(index);

        Ok(())
    }

    fn compile_continue(&mut self, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        let context = self.get_loop_context()?;

        instructions.push(Instruction::Jump(context.start));
        Ok(())
    }

    fn compile_let(&mut self, name: String, value: Option<Expression>, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        self.get_current_scope()?.add_local(name);
        if let Some(expression) = value {
            self.compile_expression(expression, instructions)?;
        } else {
            instructions.push(Instruction::PushValue(Value::Undefined));
        }

        Ok(()) 
    }

    fn compile_return(&mut self, value: Option<Expression>, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        if let Some(expression) = value {
            self.compile_expression(expression, instructions)?;
        }

        instructions.push(Instruction::Return);

        Ok(())
    }

    fn resolve_variable(&mut self, name: String) -> Result<ResolvedVar, CompilerError> {
        if let Some(index) = self.get_current_scope()?.get_local(&name) {
            Ok(ResolvedVar::Local(index))
        } else if let Some(index) = self.get_current_function_scope()?.get_upvalue(&name) {
            Ok(ResolvedVar::Upvalue(index))
        } else if let Some(index) = self.get_current_function_scope()?.add_upvalue(name.clone()) {
            Ok(ResolvedVar::Upvalue(index))
        } else {
            Err(CompilerError::UndefinedVariable(name.clone()))
        }
    }

    fn compile_assign(&mut self, target: Expression, operator: Option<BinaryOperator>, value: Expression, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        match target {
            Expression::Identifier(name) => {
                self.compile_expression(value, instructions)?;

                match self.resolve_variable(name)? {
                    ResolvedVar::Local(index) => instructions.push(Instruction::Store(index)),
                    ResolvedVar::Upvalue(index) => instructions.push(Instruction::Store(index)),
                };

                Ok(())
            },
            Expression::ArrayIndex { target, index } => {},
            Expression::Field { target, name } => {},
            _ => Err(CompilerError::InvalidAssignment)
        }
    }

    fn compile_block(&mut self, block: Vec<Statement>, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        self.push_block_scope();
        for statement in block {
            self.compile_statement(statement, instructions)?;
        }
        self.scopes.pop();
        Ok(())
    } 

    fn compile_function(&mut self, name: String, parameters: Vec<String>, block: Vec<Statement>) -> Result<(), CompilerError> {
        let mut instructions = Vec::new();
        self.push_function_scope();

        for param in &parameters {
            self.get_current_scope()?.add_local(param.to_string());
        }

        self.compile_block(block, &mut instructions)?;

        let upvalues = match self.get_current_scope()?.get_upvalues() {
            Some(upvalues) => upvalues.keys().cloned().collect(),
            None => Vec::new()
        };

        let function = CompiledFunction {
            instructions,
            parameter_count: parameters.len(),
            upvalues,
            index: self.function_prototypes.len(),
        };

        self.function_prototypes.insert(name, function);
        
        self.scopes.pop();
        Ok(())
    }

    fn compile_statement(&mut self, statement: Statement, instructions: &mut Vec<Instruction>) -> Result<(), CompilerError> {
        match statement {
            Statement::If { condition, then_branch, else_branch } => self.compile_if(condition, then_branch, else_branch, instructions)?,
            Statement::While { condition, block } => self.compile_while(condition, block, instructions)?,
            Statement::For { iterator, iterable, block } => {},
            Statement::Break => self.compile_break(instructions)?,
            Statement::Continue => self.compile_continue(instructions)?, 
            Statement::FunctionDefinition { name, parameters, block } => self.compile_function(name, parameters, block)?,
            Statement::Return(value) => self.compile_return(value, instructions)?,
            Statement::Let { name, value } => self.compile_let(name, value, instructions)?,
            Statement::Assign { target, operator, value } => self.compile_assign(target, operator, value, instructions)?,
            Statement::Match { expression, arms } => {},
            Statement::Expression(expression) => {}
        }

        Ok(())
    } 

    pub fn compiler_program(&mut self, statements: Vec<Statement>) -> Result<Vec<Instruction>, CompilerError> {
        let mut code = Vec::new();
        for statement in statements {
            self.compile_statement(statement, &mut code)?;
        }

        Ok(code)
    }
}
