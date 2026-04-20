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
            PVal::FuncLet(func) => {
                let name = func.name();

                let func = FuncDef::from(func.clone());
                table.table.insert(name, func);
            }
            PVal::Grouping(group) => {
                for stmt in group.stmts_raw() {
                    Self::visit_function(stmt, table);
                }

                if let Some(redir) = group.redirect_raw() {
                    Self::visit_function(redir, table);
                }
            }
            PVal::Match(match_) => {
                Self::visit_function(match_.expr_raw(), table);
                for arm in match_.arms_raw() {
                    Self::visit_function(&arm.expr, table);
                }
            }
            PVal::For(for_) => {
                Self::visit_function(for_.loop_raw().expr_raw(), table);
                Self::visit_function(for_.body_raw(), table);
            }
            PVal::Let(let_) => {
                Self::visit_function(let_.expr_raw(), table);
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
    scopes: Vec<HashMap<&'a str, (pest::Span<'a>, Type, String)>>,
}

impl<'a> ScopeStack<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn scope(&self) -> usize {
        self.scopes.len() - 1
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes
            .pop()
            .expect("global scope popped. you're fucked");
    }

    pub fn insert<T>(&mut self, name: &'a str, span: pest::Span<'a>, ty: Type, id: T) where T: ToString {
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name, (span, ty, id.to_string()));
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some((_, ty, _)) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    pub fn get_span(&self, name: &str) -> Option<&pest::Span<'_>> {
        for scope in self.scopes.iter().rev() {
            if let Some((span, _, _)) = scope.get(name) {
                return Some(span);
            }
        }
        None
    }

    pub fn get_unique_ident(&self, name: &str) -> Option<(Type, String)> {
        self.scopes.iter().rev().find_map(|scope| {
            scope.get(name).map(|(_, ty, unique)| (ty.clone(), unique.clone()))
        })
    }
}
