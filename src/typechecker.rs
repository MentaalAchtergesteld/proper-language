use std::collections::HashMap;

use crate::{common::{FunctionSignature, StructDefinition}, hir};

struct TypeScope {
    pub structs: HashMap<String, StructDefinition>,
    pub functions: HashMap<String, FunctionSignature>,
    pub variables: HashMap<String, Type>,
}

impl TypeScope {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }
}

pub struct TypeChecker {
    scopes: Vec<TypeScope>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        let scope = TypeScope::new();
        self.scopes.push(scope);
    }

    fn pop_scope(&mut self) -> TypeScope {
        self.scopes.pop().unwrap()
    }

    fn resolve_struct(&self, name: &str) -> Option<&StructDefinition> {
        for scope in self.scopes.iter().rev() {
            if let Some(struct_def) = scope.structs.get(name) {
                return Some(struct_def);
            }
        }
        None
    }

    fn resolve_function(&self, name: &str) -> Option<&FunctionSignature> {
        for scope in self.scopes.iter().rev() {
            if let Some(fn_def) = scope.functions.get(name) {
                return Some(fn_def);
            }
        }
        None
    }

    fn discover_definitions(&mut self, stmt_scope: Vec<hir::Statement>) {}
    fn analyze_types(&mut self, stmt_scope: Vec<hir::Statement>) {}

    pub fn check_program(&mut self, program: Vec<hir::Statement>) {}
}
