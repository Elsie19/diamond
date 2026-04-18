use std::collections::HashMap;

use crate::{
    parse::{grammar::UntypedAst, types::PVal},
    typing::{
        pass_one::{FuncDef, FuncTable},
        types::Type,
    },
};

pub struct AstWalker<'a> {
    ast: &'a UntypedAst<'a>,
}

impl<'a> AstWalker<'a> {
    pub fn new(ast: &'a UntypedAst<'a>) -> Self {
        Self { ast }
    }

    fn visit_function(node: &'a PVal<'a>, table: &mut FuncTable<'a>) {
        match node {
            PVal::Atomic(_) => {}
            PVal::FuncCall(func) => {
                Self::visit_function(func.name_raw(), table);

                if let Some(args) = func.args_raw() {
                    for arg in args.iter() {
                        Self::visit_function(arg, table);
                    }
                }
            }
            func @ PVal::FuncLet {
                name,
                args: _,
                ret: _,
                body: _,
                internal: _,
            } => {
                let func = FuncDef::try_from(func.clone());
                if let Ok(func) = func {
                    table.table.insert(
                        unsafe {
                            name.node
                                .clone()
                                .into_atomic_unchecked()
                                .node
                                .into_ident_unchecked()
                                .node
                        },
                        func,
                    );
                }
            }
            PVal::Grouping(group) => {
                for stmt in group.stmts_raw() {
                    Self::visit_function(stmt, table);
                }

                if let Some(redir) = group.redirect_raw() {
                    Self::visit_function(redir, table);
                }
            }
            PVal::Match { expr, arms } => {
                Self::visit_function(expr, table);
                for arm in arms {
                    Self::visit_function(&arm.expr, table);
                }
            }
            PVal::For(for_) => {
                Self::visit_function(&for_.loop_raw().expr_raw(), table);
                Self::visit_function(&for_.body_raw(), table);
            }
            PVal::Let { name: _, expr } => {
                Self::visit_function(expr, table);
            }
            PVal::Stmt(spanned) | PVal::Expr(spanned) => {
                Self::visit_function(spanned, table);
            }
        }
    }

    pub fn collect_function_defs(&self) -> FuncTable<'_> {
        let mut table = FuncTable::new();

        for node in self.ast {
            Self::visit_function(&node.node, &mut table);
        }

        table
    }
}

#[derive(Debug)]
pub struct ScopeStack<'a> {
    scopes: Vec<HashMap<&'a str, (pest::Span<'a>, Type)>>,
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

    pub fn insert(&mut self, name: &'a str, span: pest::Span<'a>, ty: Type) {
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name, (span, ty));
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(&ty.1);
            }
        }
        None
    }

    pub fn get_span(&self, name: &str) -> Option<&pest::Span<'_>> {
        for scope in self.scopes.iter().rev() {
            if let Some((span, _)) = scope.get(name) {
                return Some(span);
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
