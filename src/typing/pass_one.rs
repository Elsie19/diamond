//! Collect function typings.
//!
//! We don't check that they're valid just yet.

use miette::Diagnostic;
use thiserror::Error;

use crate::{parse::types::PVal, typing::types::Type};

#[derive(Debug, Error, Diagnostic)]
pub enum FuncDefConversionError {
    #[error("not a function definition")]
    NotAFuncDef,
}

pub struct FuncDef {
    args: Box<[Type]>,
    ret: Type,
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
