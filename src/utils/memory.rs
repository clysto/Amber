use heraclitus_compiler::prelude::*;
use std::collections::{HashMap, BTreeSet};
use crate::modules::{types::Type, block::Block, function::declaration_utils::FunctionDeclSyntax};
use super::{function_map::{FunctionMap, FunctionInstance}, exports::Exports, ParserMetadata};

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub returns: Type,
    pub body: Vec<Token>,
    pub meta: ParserMetadata,
    pub typed: bool,
    pub is_public: bool,
    pub id: usize
}

#[derive(Clone, Debug)]
pub struct VariableDecl {
    pub name: String,
    pub kind: Type,
    pub global_id: Option<usize>
}

#[derive(Clone, Debug)]
pub struct ScopeUnit {
    pub vars: HashMap<String, VariableDecl>,
    pub funs: HashMap<String, FunctionDecl>
}

impl ScopeUnit {
    fn new() -> ScopeUnit {
        ScopeUnit {
            vars: HashMap::new(),
            funs: HashMap::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Memory {
    pub scopes: Vec<ScopeUnit>,
    // Map of all generated functions based on their invocations
    pub function_map: FunctionMap,
    pub variable_id: usize,
    pub exports: Exports
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            scopes: vec![],
            function_map: FunctionMap::new(),
            exports: Exports::new(),
            variable_id: 0
        }
    }
    
    pub fn get_depth(&self) -> usize {
        self.scopes.len()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(ScopeUnit::new())
    }

    pub fn pop_scope(&mut self) -> Option<ScopeUnit> {
        self.scopes.pop()
    }

    pub fn add_variable(&mut self, name: &str, kind: Type, global: bool) -> Option<usize> {
        let mut global_id = None;
        if global {
            global_id = Some(self.variable_id);
            self.variable_id += 1;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.vars.insert(name.to_string(), VariableDecl {
            name: name.to_string(),
            kind,
            global_id
        });
        global_id
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableDecl> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub fn get_available_variables(&self) -> BTreeSet<&String> {
        let mut set = BTreeSet::new();
        for scope in self.scopes.iter().rev() {
            for name in scope.vars.keys() {
                set.insert(name);
            }
        }
        set
    }

    pub fn add_existing_function_declaration(&mut self, decl: FunctionDecl) -> bool {
        let scope = self.scopes.last_mut().unwrap();
        // Add function declaration to the exports
        self.exports.add_function(decl.clone());
        // Add function declaration to the scope
        let res = scope.funs.insert(decl.name.to_string(), decl);
        res.is_none()
    }

    pub fn add_function_declaration(&mut self, meta: ParserMetadata, decl: FunctionDeclSyntax) -> Option<usize> {
        let typed = !decl.args.iter().any(|(_, kind)| kind == &Type::Generic);
        let scope = self.scopes.last_mut().unwrap();
        // Add function declaration to the function map
        let id = self.function_map.add_declaration();
        // Create a new function declaration
        let function_declaration = FunctionDecl {
            name: decl.name.to_string(),
            args: decl.args.to_vec(),
            returns: decl.returns,
            is_public: decl.is_public,
            body: decl.body,
            meta,
            typed,
            id,
        };
        // Add function declaration to the scope
        let success = scope.funs.insert(decl.name, function_declaration.clone());
        // Add function declaration to the exports
        self.exports.add_function(function_declaration);
        // If this is a new function, return its id
        if success.is_none() {
            Some(id)
        }
        // If we are having a conflict
        else {
            None
        }
    }

    pub fn add_function_instance(&mut self, id: usize, args: &[Type], returns: Type, body: Block) -> usize {
        self.function_map.add_instance(id, FunctionInstance {
            args: args.to_vec(),
            returns,
            body
        })
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDecl> {
        for scope in self.scopes.iter().rev() {
            if let Some(fun) = scope.funs.get(name) {
                return Some(fun);
            }
        }
        None
    }

    pub fn get_function_instances(&self, id: usize) -> Option<&Vec<FunctionInstance>> {
        self.function_map.get(id)
    }

    pub fn set_function_map(&mut self, old_meta: &ParserMetadata) {
        self.function_map = old_meta.mem.function_map.clone();
    }

    pub fn get_available_functions(&self) -> BTreeSet<&String> {
        let mut set = BTreeSet::new();
        for scope in self.scopes.iter().rev() {
            for name in scope.funs.keys() {
                set.insert(name);
            }
        }
        set
    }
}