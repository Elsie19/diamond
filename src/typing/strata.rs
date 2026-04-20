use std::{collections::HashSet, fmt::Display};

use crate::typing::types::Type;

#[derive(Debug)]
pub struct VarGen {
    store: HashSet<String>,
}

impl Default for VarGen {
    fn default() -> Self {
        Self {
            store: HashSet::new(),
        }
    }
}

impl VarGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn var<S>(&mut self, str: S) -> &str
    where
        S: AsRef<str>,
    {
        let mut num = 0;
        loop {
            num += 1;
            let id = format!("{}_{}", Self::normalize(str.as_ref()), num);

            if self.store.insert(id.clone()) {
                return self.store.get(&id).expect("checked above");
            }
        }
    }

    fn normalize(str: &str) -> String {
        let str = str.replace(['_', '-'], "");
        str.chars().filter(|c| c.is_ascii_alphabetic()).collect()
    }
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
