use miette::Diagnostic;
use thiserror::Error;

use crate::{
    parse::{
        grammar::UntypedAst,
        types::{PAtomic, PVal, SpannedPVal},
    },
    typing::{
        core::ScopeStack,
        pass_one::{self, FuncTable},
        types::Type,
    },
};

pub struct TypeChecker<'a> {
    funcs: &'a FuncTable<'a>,
    scopes: ScopeStack<'a>,
}

#[derive(Debug, Error, Diagnostic)]
pub enum TypeCheckError {
    #[error(transparent)]
    VerifyError(pass_one::VerifyError),

    #[error("unknown function `{0}`")]
    UnknownFunction(String),

    #[error("unknown variable `{0}`")]
    UnknownVariable(String),
}

impl<'a> TypeChecker<'a> {
    pub fn new(funcs: &'a FuncTable<'a>) -> Self {
        Self {
            funcs,
            scopes: ScopeStack::new(),
        }
    }

    pub fn check_program(&mut self, program: &'a UntypedAst<'a>) -> Result<(), TypeCheckError> {
        for node in program {
            self.check_node(node)?;
        }

        Ok(())
    }

    fn check_node(&mut self, node: &'a SpannedPVal<'a>) -> Result<Type, TypeCheckError> {
        self.check_inner(&node.node, node.span())
    }

    fn check_inner(
        &mut self,
        node: &'a PVal<'a>,
        span: pest::Span<'a>,
    ) -> Result<Type, TypeCheckError> {
        match &node {
            PVal::FuncLet {
                name,
                args,
                ret,
                body,
            } => {
                self.scopes.push();

                for arg in &args.node {
                    self.scopes.insert(&arg.name, arg.ty.clone().into());
                }

                let body_ty = self.check_inner(&body.node, body.span())?;

                if let Some(ret) = ret
                    && body_ty != ret.clone().into()
                {
                    self.scopes.pop();
                    return Err(TypeCheckError::VerifyError(
                        pass_one::VerifyError::InvalidReturnType {
                            expected: Type::from(ret.clone()),
                            got: body_ty,
                        },
                    ));
                }

                self.scopes.pop();

                Ok(Type::Unit)
            }
            PVal::Atomic(spanned) => match &spanned.node {
                PAtomic::Integer(_) => Ok(Type::Integer),
                PAtomic::String(_) => Ok(Type::String),
                PAtomic::Array(spanned) => Ok(Type::Array(todo!("infer type"))),
                PAtomic::Ident(spanned) => {
                    let name = spanned.node;
                    self.scopes
                        .get(name)
                        .cloned()
                        .ok_or_else(|| TypeCheckError::UnknownVariable(name.to_string()))
                }
                PAtomic::Unit(_) => Ok(Type::Unit),
                PAtomic::Result(spanned) => todo!(),
            },
            PVal::FuncCall { name, args, unwrap } => {
                let func_name =
                    unsafe { name.node.as_atomic_unchecked().node.as_ident_unchecked() };

                let def = self
                    .funcs
                    .lookup(func_name)
                    .ok_or_else(|| TypeCheckError::UnknownFunction(func_name.to_string()))?;

                if let Some(args) = args {
                    if args.node.len() != def.args.len() {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::ArgumentLengthMismatch {
                                expected: def.args.len(),
                                got: args.node.len(),
                            },
                        ));
                    }

                    for (slot, (arg_expr, expected)) in args.node.iter().zip(&def.args).enumerate()
                    {
                        let got = self.check_node(arg_expr)?;
                        if got != *expected {
                            return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::ArgumentTypeMismatch {
                                    slot,
                                    expected: expected.clone(),
                                    got,
                                },
                            ));
                        }
                    }
                }

                let mut ret_ty = def.ret.clone();

                if *unwrap {
                    match ret_ty {
                        Type::Result(ok, _) => {
                            ret_ty = *ok;
                        }
                        _ => {
                            return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::UnwrapNonResult,
                            ));
                        }
                    }
                }

                Ok(ret_ty)
            }
            PVal::Grouping {
                stmts,
                return_expr,
                redirect,
            } => {
                self.scopes.push();

                for stmt in stmts {
                    self.check_inner(stmt, stmt.span())?;
                }

                if let Some(expr) = return_expr {
                    return self.check_inner(expr, expr.span());
                }

                if let Some(expr) = redirect {
                    return self.check_inner(expr, expr.span());
                }

                self.scopes.pop();

                Ok(Type::Unit)
            }
            PVal::Match { expr, arms } => todo!(),
            PVal::For { loop_, body } => todo!(),
            PVal::Let { name, expr } => {
                let ty = self.check_inner(expr, expr.span())?;
                self.scopes.insert(name, ty.clone());
                Ok(ty)
            }
            PVal::Rust { inner } => self.check_inner(&inner.node, inner.span()),
            PVal::Expr(spanned) | PVal::Stmt(spanned) => {
                self.check_inner(&spanned.node, spanned.span())
            }
        }
    }
}
