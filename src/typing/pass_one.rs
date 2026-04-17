//! Collect function typings & begin scoping things.
//!
//! We don't check that they're valid just yet.

use std::collections::HashMap;

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::{parse::types::PVal, typing::types::Type};

#[derive(Debug, Error, Diagnostic)]
pub enum FuncDefConversionError {
    #[error("not a function definition")]
    NotAFuncDef,
}

#[derive(Debug, Error, Diagnostic)]
pub enum VerifyError {
    #[error("expected `{}`, got `{}`", expected.as_display_ty(), got.as_display_ty())]
    #[diagnostic(code(type_checking::return_ty::is_valid::verify))]
    #[diagnostic(help("ensure the correct type is being returned"))]
    InvalidReturnType {
        expected: Type,
        got: Type,

        #[source_code]
        src: NamedSource<String>,

        #[label(primary, "wrong return type found here")]
        bad_bit: SourceSpan,

        #[label("defined here")]
        decl: SourceSpan,
    },

    #[error("cannot unwrap non-result type")]
    #[diagnostic(code(type_checking::result::is_result::verify))]
    #[diagnostic(help("remove unwrap"))]
    UnwrapNonResult {
        #[source_code]
        src: NamedSource<String>,

        #[label("erroneous unwrap found here")]
        bad_bit: SourceSpan,
    },

    #[error("expected `{}`, got `{}`", expected.as_display_ty(), got.as_display_ty())]
    #[diagnostic(code(type_checking::typing::is_valid::verify))]
    #[diagnostic(help("ensure that the correct type is being passed"))]
    MismatchedType {
        expected: Type,
        got: Type,

        #[source_code]
        src: NamedSource<String>,

        #[label("erroneous type found here")]
        bad_bit: SourceSpan,
    },

    #[error("expected `{}`, got `{}`", expected.as_display_ty(), got.as_display_ty())]
    #[diagnostic(code(type_checking::matching::homogenous::verify))]
    #[diagnostic(help("ensure that the same type is being returned by both branches"))]
    MismatchedMatchArms {
        expected: Type,
        got: Type,

        #[source_code]
        src: NamedSource<String>,

        #[label("but defined here as `{}`", expected.as_display_ty())]
        cur_branch: SourceSpan,

        #[label("previous branch defined here as `{}`", got.as_display_ty())]
        prev_branch: SourceSpan,
    },

    #[error("non-iterable expression used in iterable context")]
    #[diagnostic(code(type_checking::iter::iterable::verify))]
    #[diagnostic(help("ensure that expressions passed to loops are iterable"))]
    NonIterable {
        #[source_code]
        src: NamedSource<String>,

        #[label("non-iterable expression found here")]
        bad_bit: SourceSpan,
    },

    #[error("cannot infer type from empty array")]
    EmptyArrayInfer,

    #[error("expected `{}`, found `{}`", expected.as_display_ty(), got.as_display_ty())]
    #[diagnostic(code(type_checking::array::homogenous::verify))]
    #[diagnostic(help("ensure that the array elements are homogenously typed"))]
    MismatchedArrayElements {
        expected: Type,
        got: Type,

        #[source_code]
        src: NamedSource<String>,

        #[label("non-homogenous type found here")]
        bad_bit: SourceSpan,
    },

    #[error("expected `{expected}`, but `{got}` {} supplied", if *got == 1 { "was" } else { "were" })]
    #[diagnostic(code(type_checking::function::argument_length::verify))]
    #[diagnostic(help("ensure that the number of arguments are uniform"))]
    ArgumentLengthMismatch {
        expected: usize,
        got: usize,

        #[source_code]
        src: NamedSource<String>,

        #[label("invoked here")]
        bad_bit: SourceSpan,
    },

    #[error("expected `{}` but got `{}`", expected.as_display_ty(), got.as_display_ty())]
    #[diagnostic(code(type_checking::function::argument_check::verify))]
    #[diagnostic(help("ensure that the function parameters are correct"))]
    ArgumentTypeMismatch {
        slot: usize,
        expected: Type,
        got: Type,

        #[source_code]
        src: NamedSource<String>,

        #[label("used here")]
        bad_bit: SourceSpan,
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
                internal: _,
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
