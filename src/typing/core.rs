use crate::{
    parse::{grammar::UntypedAst, types::PVal},
    typing::pass_one::{FuncDef, FuncTable},
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
            PVal::FuncCall { name, args, .. } => {
                Self::visit_function(name, table);

                if let Some(args) = args {
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
            PVal::Grouping {
                stmts,
                return_expr,
                redirect,
            } => {
                for stmt in stmts {
                    Self::visit_function(stmt, table);
                }

                if let Some(return_) = return_expr {
                    Self::visit_function(return_, table);
                }

                if let Some(redir) = redirect {
                    Self::visit_function(redir, table);
                }
            }
            PVal::Match { expr, arms } => {
                Self::visit_function(expr, table);
                for arm in arms {
                    Self::visit_function(&arm.expr, table);
                }
            }
            PVal::For { loop_, body } => {
                Self::visit_function(&loop_.expr, table);
                Self::visit_function(body, table);
            }
            PVal::Let { name: _, expr } => {
                Self::visit_function(expr, table);
            }
            PVal::Rust { inner } => {
                Self::visit_function(inner, table);
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
