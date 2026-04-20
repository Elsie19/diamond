use crate::typing::types::Type;

/// Various ways of generating variable names.
pub mod vargen_strategies;

pub trait VarGenerator {
    /// Initialize variable generator.
    fn init() -> Self;

    /// Generate fresh variable name.
    fn fresh<S>(&mut self, str: S) -> String
    where
        S: AsRef<str>;
}

#[derive(Debug)]
pub enum IR {
    FuncLet {
        name: String,
        args: Vec<(String, Type)>,
        internal: bool,
        ret: Type,
        body: Vec<Self>,
    },
    Grouping {
        inner: Vec<Self>,
        expr_end: Option<Box<Self>>,
        redirect: Option<Box<Self>>,
    },
    For {
        bind: String,
        iter: Vec<Self>,
        body: Vec<Self>,
    },
    Let {
        name: String,
        ty: Type,
        value: Vec<Self>,
    },
    Match {
        expr: Vec<Self>,
        arms: Vec<IRMatchArm>,
    },
    FuncCall {
        name: String,
        args: Vec<Self>,
        unwrap: bool,
    },
    Integer(usize),
    String(String),
    Ident(String),
    Array(Vec<Self>),
    Unit,
    Result {
        ok: Box<Self>,
        err: Box<Self>,
    },
    Expr(Box<Self>),
    Stmt(Box<Self>),
}

#[derive(Debug)]
pub struct IRMatchArm {
    pub bind: String,
    pub is_ok: bool,
    pub body: Vec<IR>,
}
