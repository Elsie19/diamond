//! # Phase One
//!
//! ### Notes
//! 1. All function definitions are globally scoped.

use thiserror::Error;

use crate::{
    parse::types::{FuncArg, PVal},
    typing::types::Type,
};

#[derive(Debug, Error)]
pub enum PhaseOneError {
    #[error("invalid state, expected `{expected}`, got `{got}`")]
    InvalidState {
        expected: &'static str,
        got: &'static str,
    },
    #[error("invalid return type, expected `{expected:?}`, got `{got:?}`")]
    InvalidReturnType { expected: Type, got: Type },
    #[error("invalid arguments, expected `{expected:?}`, got `{got:?}`")]
    InvalidArgs { expected: Type, got: Type },
}

pub struct FuncDef<'a> {
    name: &'a str,
    args: Box<[FuncArg<'a>]>,
    ret: Type,
}

impl<'a> FuncDef<'a> {
    pub fn from_pval(func: PVal<'a>) -> Result<Self, PhaseOneError> {
        match func {
            PVal::FuncLet {
                name, args, ret, ..
            } => Ok(Self {
                name: unsafe {
                    name.into_inner()
                        .into_atomic_unchecked()
                        .node
                        .into_string_unchecked()
                        .node
                },
                args: args.into_inner(),
                ret: match ret {
                    Some(o) => Type::from(o),
                    None => Type::default(),
                },
            }),
            err => Err(PhaseOneError::InvalidState {
                expected: "FuncLet",
                got: err.into_name(),
            }),
        }
    }

    pub fn verify(&self, rhs: &Self) -> Result<(), PhaseOneError> {
        let Self { name, args, ret } = rhs;

        if self.ret != *ret {
            return Err(PhaseOneError::InvalidReturnType {
                expected: self.ret.clone(),
                got: ret.clone(),
            });
        }

        Ok(())
    }
}
