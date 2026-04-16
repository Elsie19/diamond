//! Collect function typings & begin scoping things.
//!
//! We don't check that they're valid just yet.

use std::collections::HashMap;

use miette::Diagnostic;
use thiserror::Error;

use crate::{parse::types::PVal, typing::types::Type};

#[derive(Debug, Error, Diagnostic)]
pub enum FuncDefConversionError {
    #[error("not a function definition")]
    NotAFuncDef,
}

// TODO: I NEED SPANS HERE FOR DIAGNOSTICS!!!
#[derive(Debug, Error, Diagnostic)]
pub enum VerifyError {
    #[error("expected {expected:?}, got {got:?}")]
    InvalidReturnType { expected: Type, got: Type },

    #[error("cannot unwrap non-result type")]
    UnwrapNonResult,

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
    pub args: Box<[Type]>,
    pub ret: Type,
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
