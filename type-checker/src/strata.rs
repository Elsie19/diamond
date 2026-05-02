use std::rc::Rc;

use bincode::{Decode, Encode};

use crate::types::Type;

/// Various ways of generating variable names.
pub mod vargen_strategies;

pub trait VarGenerator {
    /// Initialize variable generator.
    fn init() -> Self;

    /// Generate fresh variable name.
    fn fresh(&mut self) -> usize;
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum IR {
    FuncLet {
        name: Rc<str>,
        args: Vec<(usize, Type)>,
        internal: bool,
        ret: Type,
        body: Box<Self>,
    },
    Grouping {
        inner: Vec<Self>,
        /// Expression and binding name.
        redirect: Option<(Box<Self>, usize)>,
    },
    For {
        bind: usize,
        iter: Box<Self>,
        body: Box<Self>,
    },
    Let {
        name: usize,
        ty: Type,
        value: Box<Self>,
    },
    Match {
        expr: Box<Self>,
        arms: Vec<IRMatchArm>,
    },
    FuncCall {
        name: Rc<str>,
        args: Vec<Self>,
        unwrap: bool,
    },
    Integer(usize),
    String(Rc<str>),
    Ident(usize),
    Array(Vec<Self>),
    Unit,
    Result {
        ok: Box<Self>,
        err: Box<Self>,
    },
    Expr(Box<Self>),
    Stmt(Box<Self>),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct IRMatchArm {
    pub bind: usize,
    pub is_ok: bool,
    pub body: Box<IR>,
}
