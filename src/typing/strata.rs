use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::typing::types::Type;

/// Various ways of generating variable names.
pub mod vargen_strategies;

pub trait VarGenerator {
    /// Initialize variable generator.
    fn init() -> Self;

    /// Generate fresh variable name.
    fn fresh<S>(&mut self, str: S) -> Rc<str>
    where
        S: AsRef<str>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IR {
    FuncLet {
        name: Rc<str>,
        args: Vec<(Rc<str>, Type)>,
        internal: bool,
        ret: Type,
        body: Rc<Self>,
    },
    Grouping {
        inner: Vec<Self>,
        /// Expression and binding name.
        redirect: Option<(Box<Self>, Rc<str>)>,
    },
    For {
        bind: Rc<str>,
        iter: Rc<Self>,
        body: Rc<Self>,
    },
    Let {
        name: Rc<str>,
        ty: Type,
        value: Rc<Self>,
    },
    Match {
        expr: Rc<Self>,
        arms: Vec<IRMatchArm>,
    },
    FuncCall {
        name: Rc<str>,
        args: Vec<Self>,
        unwrap: bool,
    },
    Integer(usize),
    String(Rc<str>),
    Ident(Rc<str>),
    Array(Vec<Self>),
    Unit,
    Result {
        ok: Box<Self>,
        err: Box<Self>,
    },
    Expr(Rc<Self>),
    Stmt(Rc<Self>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRMatchArm {
    pub bind: Rc<str>,
    pub is_ok: bool,
    pub body: Box<IR>,
}
