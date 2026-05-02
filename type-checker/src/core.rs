use std::collections::HashMap;

use parse::{grammar::UntypedAst, types::PVal};

use crate::{
    pass_one::{FuncDef, FuncTable},
    types::Type,
};

#[derive(Debug)]
pub struct AstWalker<'a> {
    ast: &'a UntypedAst<'a>,
}

impl<'a> AstWalker<'a> {
    pub fn new(ast: &'a UntypedAst<'a>) -> Self {
        Self { ast }
    }

    fn visit_function(node: &'a PVal<'a>, table: &mut FuncTable) {
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
                table.table.insert(name.into(), func);
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

    pub fn collect_function_defs(&self) -> FuncTable {
        let mut table = FuncTable::new();

        for node in self.ast {
            Self::visit_function(&node.node, &mut table);
        }

        table
    }
}

#[derive(Debug)]
pub struct ScopeStack<'a> {
    scopes: Vec<HashMap<&'a str, (pest::Span<'a>, Type, usize)>>,
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

    pub fn insert(&mut self, name: &'a str, span: pest::Span<'a>, ty: Type, id: usize) {
        let scope = unsafe { self.scopes.last_mut().unwrap_unchecked() };
        scope.insert(name, (span, ty, id));
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|(_, ty, _)| ty))
    }

    pub fn get_span(&self, name: &str) -> Option<&pest::Span<'_>> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|(span, _, _)| span))
    }

    pub fn get_unique_ident(&self, name: &str) -> Option<(Type, usize)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|(_, ty, unique)| (ty.clone(), *unique)))
    }
}
