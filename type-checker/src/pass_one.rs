//! Collect function typings & begin scoping things.
//!
//! We don't check that they're valid just yet.

use std::{collections::HashMap, rc::Rc};

use bincode::{Decode, Encode};
use miette::{Diagnostic, NamedSource, SourceSpan};
use parse::types::funclet::FuncLet;
use thiserror::Error;

use crate::types::Type;

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

        #[label(primary, "`{}` returned here", got.as_display_ty())]
        bad_bit: SourceSpan,

        #[label("but was defined to return `{}` here", expected.as_display_ty())]
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

        #[label("but here returns `{}`", expected.as_display_ty())]
        cur_branch: SourceSpan,

        #[label("previous branch returns `{}`", got.as_display_ty())]
        prev_branch: SourceSpan,
    },

    #[error("non-iterable expression (a `{}`) used in iterable context", got.as_display_ty())]
    #[diagnostic(code(type_checking::iter::iterable::verify))]
    #[diagnostic(help("ensure that expressions passed to loops are iterable"))]
    NonIterable {
        #[source_code]
        src: NamedSource<String>,

        #[label("non-iterable expression found here")]
        bad_bit: SourceSpan,

        #[label("defined here")]
        defined_here: Option<SourceSpan>,

        got: Type,
    },

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

    #[error("expected an argument count of `{expected}`, but `{got}` {} supplied", if *got == 1 { "was" } else { "were" })]
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

        #[label("{} used here", got.as_display_ty())]
        bad_bit: SourceSpan,

        #[label("defined here")]
        defined_here: Option<SourceSpan>,
    },
}

#[derive(Debug, Default, Decode, Encode)]
pub struct FuncTable {
    pub table: HashMap<Rc<str>, FuncDef>,
}

#[derive(Debug, Decode, Encode)]
pub struct FuncDef {
    pub args: Box<[Type]>,
    pub ret: Type,
}

impl From<FuncLet<'_>> for FuncDef {
    fn from(value: FuncLet<'_>) -> Self {
        let (possible_args, ret) = value.into_args_ret();

        Self {
            args: possible_args
                .map(|args| {
                    args.into_iter()
                        .map(|arg_pair| arg_pair.ty.into())
                        .collect()
                })
                .unwrap_or_default(),
            ret: ret.map_or(Type::default(), Into::into),
        }
    }
}

impl FuncTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lookup(&self, name: &str) -> Option<&FuncDef> {
        self.table.get(name)
    }

    pub fn lookup_ret(&self, name: &str) -> Option<&Type> {
        self.table.get(name).map(|val| &val.ret)
    }

    pub fn extend(&mut self, rhs: Self) {
        self.table.extend(rhs.table);
    }
}
