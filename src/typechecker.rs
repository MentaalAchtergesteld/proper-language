use std::collections::HashMap;

use crate::{common::{EnumDefinition, FunctionSignature, StructDefinition, TraitDefinition}, hir};

enum Type {
    Integer,
    Float,
    Bool,
    String,
    Unit,
    Never,

    Function {
        params: Vec<Type>,
        return_type: Box<Type>
    },
    Struct(StructDefinition),
    Enum(EnumDefinition),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Generic(String),
    Unknown
}

struct TypeScope {
    pub structs: HashMap<String, StructDefinition>,
    pub functions: HashMap<String, FunctionSignature>,
    pub traits: HashMap<String, TraitDefinition>,
    pub enums: HashMap<String, EnumDefinition>,
    pub variables: HashMap<String, Type>,
}

impl TypeScope {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            functions: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
            variables: HashMap::new(),
        }
    }
}

pub enum TypeCheckError {
    StructAlreadyDefined(String),
    FunctionAlreadyDefined(String),
    EnumAlreadyDefined(String),
    TraitAlreadyDefined(String),
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

    fn current_scope(&self) -> &TypeScope {
        self.scopes.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut TypeScope {
        self.scopes.last_mut().unwrap()
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

    fn discover_scope(&mut self, stmt_scope: &[hir::Statement]) -> Result<(), TypeCheckError> {
        for stmt in stmt_scope {
            match stmt {
                hir::Statement::EnumDefinition(def) => {
                    let scope = self.current_scope_mut();
                    if scope.enums.contains_key(&def.name) {
                        return Err(TypeCheckError::EnumAlreadyDefined(def.name.clone()));
                    }

                    scope.enums.insert(def.name.clone(), def.clone());
                },
                hir::Statement::TraitDefinition(def) => {
                    let scope = self.current_scope_mut();
                    if scope.traits.contains_key(&def.name) {
                        return Err(TypeCheckError::TraitAlreadyDefined(def.name.clone()));
                    }
                    
                    scope.traits.insert(def.name.clone(), def.clone());
                },
                hir::Statement::StructDefinition(def) => {
                    let scope = self.current_scope_mut();
                    if scope.structs.contains_key(&def.name) {
                        return Err(TypeCheckError::StructAlreadyDefined(def.name.clone()));
                    }

                    scope.structs.insert(def.name.clone(), def.clone());
                },
                hir::Statement::FunctionDefinition(def) => {
                    let signature = &def.signature;
                    let scope = self.current_scope_mut();
                    if scope.functions.contains_key(&signature.name) {
                        return Err(TypeCheckError::FunctionAlreadyDefined(signature.name.clone()));
                    }

                    scope.functions.insert(signature.name.clone(), signature.clone());
                },
                _ => {}
            }
        } 

        for stmt in stmt_scope {
            if let hir::Statement::ImplBlock(block) = &stmt {
            }
        }

        Ok(())
    }

    fn analyze_types(&mut self, stmt_scope: Vec<hir::Statement>) {}

    pub fn check_program(&mut self, program: Vec<hir::Statement>) {}
}
