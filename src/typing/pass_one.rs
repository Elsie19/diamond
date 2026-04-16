//! Collect function typings & begin scoping things.
//!
//! We don't check that they're valid just yet.

use std::collections::HashMap;

use miette::Diagnostic;
use thiserror::Error;

use crate::{
    parse::types::{PType, PVal},
    typing::types::Type,
};

#[derive(Debug, Error, Diagnostic)]
pub enum FuncDefConversionError {
    #[error("not a function definition")]
    NotAFuncDef,
}

#[derive(Debug, Error, Diagnostic)]
pub enum VerifyError {
    #[error("expected {expected:?}, got {got:?}")]
    InvalidReturnType { expected: Type, got: Type },

    #[error("expected {expected:?}, got {got:?}")]
    ArgumentLengthMismatch { expected: usize, got: usize },

    #[error("expected {expected:?} but got {got:?} at pos {slot}")]
    ArgumentTypeMismatch {
        slot: usize,
        expected: Type,
        got: Type,
    },
}

#[derive(Debug)]
pub struct FuncTable<'a> {
    pub table: HashMap<&'a str, FuncDef>,
}

#[derive(Debug)]
pub struct FuncDef {
    args: Box<[Type]>,
    ret: Type,
}

pub struct ScopeStack<'a> {
    scopes: Vec<HashMap<&'a str, PType<'a>>>,
}

impl<'a> ScopeStack<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes
            .pop()
            .expect("global scope popped. you're fucked");
    }

    pub fn insert(&mut self, name: &'a str, ty: PType<'a>) {
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<&PType<'_>> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    pub fn with_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.push();
        let result = f(self);
        self.pop();
        result
    }
}

impl TryFrom<PVal<'_>> for FuncDef {
    type Error = FuncDefConversionError;

    fn try_from(value: PVal<'_>) -> Result<Self, Self::Error> {
        match value {
            PVal::FuncLet {
                name: _,
                args,
                ret,
                body: _,
            } => Ok(Self {
                args: args
                    .into_iter()
                    .map(|arg_pair| arg_pair.ty.into())
                    .collect(),
                ret: ret.map_or(Type::default(), Into::into),
            }),
            _ => Err(FuncDefConversionError::NotAFuncDef),
        }
    }
}

impl FuncDef {
    pub fn verify(&self, rhs: &Self) -> Result<(), VerifyError> {
        Ok(())
    }
}

impl FuncTable<'_> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&FuncDef> {
        self.table.get(name)
    }
}
