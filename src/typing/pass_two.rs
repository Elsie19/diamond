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
    source: NamedSource<String>,
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
            source: NamedSource::new(file_name, prog_text.to_string()).with_language("diamond"),
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
                let expected = ret.clone().map_or(Type::Unit, Type::from);

                if *internal {
                    return Ok(expected);
                }

                self.scopes.push();

                for arg in &args.node {
                    self.scopes
                        .insert(&arg.name, arg.name.span(), arg.ty.clone().into());
                }

                let result = {
                    let got = self.check_inner(&body.node, body.span())?;
                    if got != expected {
                        Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::InvalidReturnType {
                                expected,
                                got,
                                src: self.source.clone(),
                                bad_bit: body.as_miette_span(),
                                decl: spest_to_smiette(ret.clone().unwrap().span()),
                            },
                        ))
                    } else {
                        Ok(expected)
                    }
                };

                self.scopes.pop();
                result
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
                        src: self.source.clone(),
                        bad_bit: spest_to_smiette(span),
                    }
                })?;

                let args = args.as_ref().map(|a| &a.node[..]).unwrap_or(&[]);

                if args.len() != def.args.len() {
                    return Err(TypeCheckError::VerifyError(
                        pass_one::VerifyError::ArgumentLengthMismatch {
                            expected: def.args.len(),
                            got: args.len(),
                            src: self.source.clone(),
                            bad_bit: spest_to_smiette(span),
                        },
                    ));
                }

                for (slot, (arg_expr, expected)) in args.iter().zip(&def.args).enumerate() {
                    let got = self.check_node(arg_expr)?;
                    if got != *expected {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::ArgumentTypeMismatch {
                                slot,
                                expected: expected.clone(),
                                got,
                                src: self.source.clone(),
                                bad_bit: arg_expr.as_miette_span(),
                            },
                        ));
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
                                    src: self.source.clone(),
                                    bad_bit: unwrap.as_miette_span(),
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
                    let got = self.check_inner(expr, expr.span())?;
                    if !matches!(got, Type::Stream) {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::MismatchedType {
                                expected: Type::Stream,
                                got,
                                src: self.source.clone(),
                                bad_bit: expr.as_miette_span(),
                            },
                        ));
                    }
                }

                let result = if let Some(expr) = return_expr {
                    self.check_inner(expr, expr.span())?
                } else {
                    /*
                     * Remember that:
                     * {
                     *  foo;
                     *  bar;
                     *  baz;
                     * }
                     *
                     * the last expression isn't actually one, it's a statement.
                     */
                    Type::Unit
                };

                self.scopes.pop();

                Ok(result)
            }
            PVal::Match { expr, arms } => {
                let expr_ty = self.check_inner(expr, expr.span())?;

                let (ok_ty, err_ty) = match expr_ty {
                    Type::Result(ok, err) => (*ok, *err),
                    _ => {
                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::UnwrapNonResult {
                                src: self.source.clone(),
                                bad_bit: expr.as_miette_span(),
                            },
                        ));
                    }
                };

                let mut result_ty: Option<Type> = None;
                let mut last_span = None;

                for arm in arms {
                    self.scopes.push();

                    let cur = match &arm.res {
                        PMatchCase::Ok(_) => {
                            self.scopes
                                .insert(&arm.inner, arm.inner.span(), ok_ty.clone());
                            arm.expr.span()
                        }
                        PMatchCase::Err(_) => {
                            self.scopes
                                .insert(&arm.inner, arm.inner.span(), err_ty.clone());
                            arm.expr.span()
                        }
                    };

                    let arm_ty = self.check_inner(&arm.expr.node, arm.expr.span())?;

                    self.scopes.pop();

                    if let Some(prev) = &result_ty {
                        if *prev != arm_ty {
                            return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::MismatchedMatchArms {
                                    expected: arm_ty,
                                    got: prev.clone(),
                                    src: self.source.clone(),
                                    cur_branch: spest_to_smiette(cur),
                                    prev_branch: spest_to_smiette(last_span.expect("how are we failing on a current branch if we don't have a previous")),
                                },
                            ));
                        }
                    } else {
                        last_span = Some(cur);
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
                        // We can get a little clever here. If what's trying to be used as an
                        // iterable is not a constant, but an identifier, we can go find its span
                        // and have even nicer error messages.
                        let defined_here = match &*loop_.expr.node {
                            PVal::Atomic(spanned) => match &spanned.node {
                                PAtomic::Ident(name) => self
                                    .scopes
                                    .get_span(name.node)
                                    .map(|span| spest_to_smiette(*span)),
                                _ => None,
                            },
                            _ => None,
                        };

                        return Err(TypeCheckError::VerifyError(
                            pass_one::VerifyError::NonIterable {
                                src: self.source.clone(),
                                bad_bit: loop_.expr.as_miette_span(),
                                defined_here,
                            },
                        ));
                    }
                };

                self.scopes.push();

                self.scopes.insert(&loop_.bind, loop_.bind.span(), elem_ty);

                let for_ret_ty = self.check_inner(&body.node, body.span())?;

                self.scopes.pop();

                Ok(for_ret_ty)
            }
            PVal::Let { name, expr } => {
                let ty = self.check_inner(expr, expr.span())?;

                self.scopes.insert(name, name.span(), ty.clone());
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
                        let expected = self.check_node(first)?;
                        for elem in rest {
                            let got = self.check_node(elem)?;

                            if got != expected {
                                return Err(TypeCheckError::VerifyError(
                                    pass_one::VerifyError::MismatchedArrayElements {
                                        expected,
                                        got,
                                        src: self.source.clone(),
                                        bad_bit: elem.as_miette_span(),
                                    },
                                ));
                            }
                        }
                        Ok(Type::Array(Box::new(expected)))
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
                        src: self.source.clone(),
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
