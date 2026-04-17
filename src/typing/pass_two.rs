use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::{
    parse::{
        grammar::{UntypedAst, spest_to_smiette},
        types::{PAtomic, PMatchCase, PVal, SpannedPVal},
    },
    typing::{
        core::ScopeStack,
        pass_one::{self, FuncTable},
        types::Type,
    },
};

#[derive(Debug)]
pub struct TypeChecker<'a> {
    funcs: &'a FuncTable<'a>,
    scopes: ScopeStack<'a>,
    file_name: &'a str,
    prog_text: &'a str,
}

#[derive(Debug, Error, Diagnostic)]
pub enum TypeCheckError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    VerifyError(pass_one::VerifyError),

    #[error("unknown function `{name}`")]
    #[diagnostic(code(type_checking::function::verify_exists))]
    #[diagnostic(help("ensure that the function exists"))]
    UnknownFunction {
        name: String,

        #[source_code]
        src: NamedSource<String>,

        #[label("invoked here")]
        bad_bit: SourceSpan,
    },

    #[error("unknown variable `{name}`")]
    #[diagnostic(code(type_checking::variable::verify_exists))]
    #[diagnostic(help("ensure that the correct variable name is used"))]
    UnknownVariable {
        name: String,

        #[source_code]
        src: NamedSource<String>,

        #[label(primary, "used here")]
        bad_bit: SourceSpan,
    },
}

impl<'a> TypeChecker<'a> {
    pub fn new(funcs: &'a FuncTable<'a>, file_name: &'a str, prog_text: &'a str) -> Self {
        Self {
            funcs,
            scopes: ScopeStack::new(),
            file_name,
            prog_text,
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
                name: _,
                args,
                ret,
                body,
                internal,
            } => {
                if *internal {
                    return Ok(Type::Unit);
                }

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
                int @ PAtomic::Integer(_) => self.check_atomic(int, int.span()),
                string @ PAtomic::String(_) => self.check_atomic(string, string.span()),
                arr @ PAtomic::Array(_) => self.check_atomic(arr, arr.span()),
                ident @ PAtomic::Ident(_) => self.check_atomic(ident, ident.span()),
                unit @ PAtomic::Unit(_) => self.check_atomic(unit, unit.span()),
                res @ PAtomic::Result(_) => self.check_atomic(res, res.span()),
            },
            PVal::FuncCall { name, args, unwrap } => {
                let func_name =
                    unsafe { name.node.as_atomic_unchecked().node.as_ident_unchecked() };

                let def = self.funcs.lookup(func_name).ok_or_else(|| {
                    TypeCheckError::UnknownFunction {
                        name: func_name.to_string(),
                        src: NamedSource::new(self.file_name, self.prog_text.to_string())
                            .with_language("diamond"),
                        bad_bit: spest_to_smiette(span),
                    }
                })?;

                if let Some(args) = args {
                    if args.node.len() != def.args.len() {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::ArgumentLengthMismatch {
                                expected: def.args.len(),
                                got: args.node.len(),
                                src: NamedSource::new(self.file_name, self.prog_text.to_string())
                                    .with_language("diamond"),
                                bad_bit: spest_to_smiette(span),
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
                                    src: NamedSource::new(
                                        self.file_name,
                                        self.prog_text.to_string(),
                                    )
                                    .with_language("diamond"),
                                    bad_bit: spest_to_smiette(arg_expr.span()),
                                },
                            ));
                        }
                    }
                }

                let mut ret_ty = def.ret.clone();

                if let Some(unwrap) = unwrap {
                    match ret_ty {
                        Type::Result(ok, _) => {
                            ret_ty = *ok;
                        }
                        _ => {
                            return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::UnwrapNonResult {
                                    src: NamedSource::new(
                                        self.file_name,
                                        self.prog_text.to_string(),
                                    )
                                    .with_language("diamond"),
                                    bad_bit: spest_to_smiette(unwrap.span()),
                                },
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
                    self.check_node(stmt)?;
                }

                if let Some(expr) = redirect {
                    let res = self.check_inner(expr, expr.span());
                    dbg!(&res);
                    if !matches!(res, Ok(Type::Stream)) {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::ExpectedStream {
                                src: NamedSource::new(self.file_name, self.prog_text.to_string())
                                    .with_language("diamond"),
                                bad_bit: spest_to_smiette(expr.span()),
                            },
                        ));
                    }
                }

                self.scopes.pop();

                if let Some(expr) = return_expr {
                    return self.check_inner(expr, expr.span());
                }

                Ok(Type::Unit)
            }
            PVal::Match { expr, arms } => {
                let expr_ty = self.check_inner(expr, expr.span())?;

                let (ok_ty, err_ty) = match expr_ty {
                    Type::Result(ok, err) => (*ok, *err),
                    _ => {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::UnwrapNonResult {
                                src: NamedSource::new(self.file_name, self.prog_text.to_string())
                                    .with_language("diamond"),
                                bad_bit: spest_to_smiette(expr.span()),
                            },
                        ));
                    }
                };

                let mut result_ty = None;

                for arm in arms {
                    self.scopes.push();

                    match arm.res {
                        PMatchCase::Ok(_) => {
                            self.scopes.insert(&arm.inner, ok_ty.clone());
                        }
                        PMatchCase::Err(_) => {
                            self.scopes.insert(&arm.inner, err_ty.clone());
                        }
                    }

                    let arm_ty = self.check_inner(&arm.expr.node, arm.expr.span())?;

                    self.scopes.pop();

                    if let Some(prev) = &result_ty {
                        if *prev != arm_ty {
                            return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::MismatchedMatchArms,
                            ));
                        }
                    } else {
                        result_ty = Some(arm_ty);
                    }
                }

                Ok(result_ty.unwrap_or_default())
            }
            PVal::For { loop_, body } => {
                let iter_ty = self.check_inner(&loop_.expr.node, loop_.expr.span())?;

                let elem_ty = match iter_ty {
                    Type::Array(inner) => *inner,
                    _ => {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::NonIterable {
                                src: NamedSource::new(self.file_name, self.prog_text.to_string())
                                    .with_language("diamond"),
                                bad_bit: spest_to_smiette(loop_.expr.span()),
                            },
                        ));
                    }
                };

                self.scopes.push();

                self.scopes.insert(&loop_.bind, elem_ty);

                self.check_inner(&body.node, body.span())?;

                self.scopes.pop();

                Ok(Type::Unit)
            }
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

    fn check_atomic(
        &mut self,
        atom: &'a PAtomic<'a>,
        span: pest::Span<'a>,
    ) -> Result<Type, TypeCheckError> {
        match atom {
            PAtomic::Integer(_) => Ok(Type::Integer),
            PAtomic::String(_) => Ok(Type::String),
            PAtomic::Array(spanned) => {
                let elems = &spanned.node.node;

                match elems.iter().as_slice() {
                    [] => Err(TypeCheckError::VerifyError(
                        pass_one::VerifyError::EmptyArrayInfer,
                    )),
                    [first, rest @ ..] => {
                        let first_ty = self.check_node(first)?;
                        for elem in rest {
                            let ty = self.check_node(elem)?;

                            if ty != first_ty {
                                return Err(TypeCheckError::VerifyError(
                                    pass_one::VerifyError::MismatchedArrayElements {
                                        expected: first_ty,
                                        got: ty,
                                        src: NamedSource::new(
                                            self.file_name,
                                            self.prog_text.to_string(),
                                        )
                                        .with_language("diamond"),
                                        bad_bit: spest_to_smiette(elem.span()),
                                    },
                                ));
                            }
                        }
                        Ok(Type::Array(Box::new(first_ty)))
                    }
                }
            }
            PAtomic::Ident(ident) => {
                let name = ident.node;
                self.scopes
                    .get(name)
                    .cloned()
                    .ok_or_else(|| TypeCheckError::UnknownVariable {
                        name: name.to_string(),
                        src: NamedSource::new(self.file_name, self.prog_text.to_string())
                            .with_language("diamond"),
                        bad_bit: spest_to_smiette(span),
                    })
            }
            PAtomic::Unit(_) => Ok(Type::Unit),
            PAtomic::Result(spanned) => Ok(Type::Result(
                Box::new(self.check_atomic(&spanned.0, spanned.0.span())?),
                Box::new(self.check_atomic(&spanned.1, spanned.1.span())?),
            )),
        }
    }
}
