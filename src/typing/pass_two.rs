use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::{
    parse::{
        grammar::{MietteSpan, UntypedAst, spest_to_smiette},
        types::{
            PAtomic, PVal, Spanned, SpannedPVal,
            for_::For,
            funccall::FuncCall,
            funclet::FuncLet,
            grouping::Grouping,
            let_::Let,
            match_::{Match, PMatchCase},
        },
    },
    typing::{
        core::ScopeStack,
        pass_one::{self, FuncTable},
        strata::{IR, IRMatchArm, VarGenerator},
        types::Type,
    },
};

#[derive(Debug)]
pub struct TypeChecker<'a, G> {
    funcs: &'a FuncTable<'a>,
    scopes: ScopeStack<'a>,
    source: NamedSource<String>,
    var_gen: G,
    ir: Vec<IR>,
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

impl<'a, G> TypeChecker<'a, G>
where
    G: VarGenerator,
{
    pub fn new<T>(funcs: &'a FuncTable<'a>, file_name: &'a str, prog_text: &T) -> Self
    where
        T: ToString,
    {
        Self {
            funcs,
            scopes: ScopeStack::new(),
            source: NamedSource::new(file_name, prog_text.to_string()).with_language("diamond"),
            var_gen: G::init(),
            ir: Vec::new(),
        }
    }

    fn src(&self) -> NamedSource<String> {
        self.source.clone()
    }

    fn span(&self, span: pest::Span<'a>) -> SourceSpan {
        spest_to_smiette(span)
    }

    pub fn ir(&self) -> &[IR] {
        &self.ir
    }

    pub fn check_program(&mut self, program: &'a UntypedAst<'a>) -> Result<&[IR], TypeCheckError> {
        for node in program {
            self.check_node(node)?;
        }

        Ok(&self.ir)
    }

    fn inner(&mut self, sp: &'a Spanned<'a, Box<PVal<'a>>>) -> Result<Type, TypeCheckError> {
        self.check_inner(&sp.node, sp.span())
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
            PVal::FuncLet(funclet) => self.check_funclet(funclet),
            PVal::Atomic(spanned) => self.check_atomic(&spanned.node),
            PVal::FuncCall(func) => self.check_funccall(func, span),
            PVal::Grouping(group) => self.check_grouping(group),
            PVal::Match(match_) => self.check_match(match_),
            PVal::For(for_) => self.check_for(for_),
            PVal::Let(let_) => self.check_let(let_),
            PVal::Expr(spanned) => self.check_inner(&spanned.node, spanned.span()),
            // Type check everything inside but it still returns a [`Type::Unit`] by design.
            PVal::Stmt(spanned) => {
                let _ = self.check_inner(&spanned.node, spanned.span())?;
                Ok(Type::Unit)
            }
        }
    }

    fn check_for(&mut self, for_: &'a For<'a>) -> Result<Type, TypeCheckError> {
        let loop_expr = for_.loop_raw().expr_raw();

        let prev_iter_len = self.ir.len();
        let iter_ty = self.inner(loop_expr)?;

        let iter_ir = self.ir.drain(prev_iter_len..).collect::<Vec<_>>();

        let elem_ty = if let Type::Array(inner) = iter_ty {
            *inner
        } else {
            // We can get a little clever here. If what's trying to be used as an
            // iterable is not a constant, but an identifier, we can go find its span
            // and have even nicer error messages.
            let defined_here = match &***loop_expr {
                PVal::Atomic(spanned) => match &spanned.node {
                    PAtomic::Ident(name) => {
                        self.scopes.get_span(name.node).map(|span| self.span(*span))
                    }
                    _ => None,
                },
                _ => None,
            };

            return Err(TypeCheckError::VerifyError(
                pass_one::VerifyError::NonIterable {
                    src: self.src(),
                    bad_bit: loop_expr.as_miette_span(),
                    defined_here,
                },
            ));
        };

        self.scopes.push();

        let bind_name = for_.loop_raw().bind_raw();

        let unique = self.var_gen.fresh(**bind_name);

        let bind = unique.to_string();

        self.scopes
            .insert(bind_name.node, bind_name.span(), elem_ty, unique);

        let prev_body_len = self.ir.len();
        let for_ret_ty = self.inner(for_.body_raw())?;
        let body_ir = self.ir.drain(prev_body_len..).collect::<Vec<_>>();

        self.scopes.pop();

        self.ir.push(IR::For {
            bind,
            iter: iter_ir,
            body: body_ir,
        });

        Ok(for_ret_ty)
    }

    fn check_match(&mut self, match_: &'a Match<'a>) -> Result<Type, TypeCheckError> {
        let expr_raw = match_.expr_raw();

        let prev_expr_len = self.ir.len();
        let expr_ty = self.inner(expr_raw)?;
        let expr_ir = self.ir.drain(prev_expr_len..).collect::<Vec<_>>();

        let (ok_ty, err_ty) = match expr_ty {
            Type::Result(ok, err) => (*ok, *err),
            _ => {
                return Err(TypeCheckError::VerifyError(
                    pass_one::VerifyError::UnwrapNonResult {
                        src: self.src(),
                        bad_bit: expr_raw.as_miette_span(),
                    },
                ));
            }
        };

        let mut result_ty: Option<Type> = None;
        let mut last_span = None;
        let mut arms_ir = vec![];

        for arm in match_.arms_raw() {
            self.scopes.push();

            let prev_arm_len = self.ir.len();

            let (bind_ty, is_ok) = match &arm.res {
                PMatchCase::Ok(_) => (ok_ty.clone(), true),
                PMatchCase::Err(_) => (err_ty.clone(), false),
            };

            let unique = self.var_gen.fresh(arm.res.name());

            let unique = unique.to_string();

            self.scopes
                .insert(&arm.inner, arm.inner.span(), bind_ty, &unique);

            let cur = arm.expr.span();

            let expected = self.inner(&arm.expr)?;
            let body_ir = self.ir.drain(prev_arm_len..).collect::<Vec<_>>();

            self.scopes.pop();

            arms_ir.push(IRMatchArm {
                bind: unique,
                is_ok,
                body: body_ir,
            });

            if let Some(got) = &result_ty {
                if *got != expected {
                    let got = got.clone();
                    return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::MismatchedMatchArms {
                                    expected,
                                    got,
                                    src: self.src(),
                                    cur_branch: self.span(cur),
                                    prev_branch: self.span(last_span.expect("how are we failing on a current branch if we don't have a previous")),
                                },
                            ));
                }
            } else {
                last_span = Some(cur);
                result_ty = Some(expected);
            }
        }

        self.ir.push(IR::Match {
            expr: expr_ir,
            arms: arms_ir,
        });

        Ok(result_ty.unwrap_or_default())
    }

    fn check_let(&mut self, let_: &'a Let<'a>) -> Result<Type, TypeCheckError> {
        let prev_len = self.ir.len();
        let ty = self.inner(let_.expr_raw())?;

        let expr_ir = self.ir.drain(prev_len..).collect::<Vec<_>>();

        debug_assert_eq!(expr_ir.len(), 1);

        let name = let_.name_raw();
        let unique = self.var_gen.fresh(**name);

        self.scopes.insert(name, name.span(), ty.clone(), &unique);

        self.ir.push(IR::Let {
            name: unique,
            ty: ty.clone(),
            value: expr_ir,
        });

        Ok(ty)
    }

    fn check_funclet(&mut self, funclet: &'a FuncLet<'a>) -> Result<Type, TypeCheckError> {
        let expected = funclet.ret().cloned().map_or(Type::Unit, Type::from);

        if funclet.is_internal() {
            return Ok(expected);
        }

        self.scopes.push();

        if let Some(args) = &funclet.args_raw() {
            for arg in &args.node {
                self.scopes.insert(
                    &arg.name,
                    arg.name.span(),
                    arg.ty.clone().into(),
                    self.var_gen.fresh(*arg.name),
                );
            }
        }

        let prev_len = self.ir.len();

        let body = funclet.body_raw();
        let got = self.inner(body)?;

        if got != expected {
            self.scopes.pop();
            return Err(TypeCheckError::VerifyError(
                pass_one::VerifyError::InvalidReturnType {
                    expected,
                    got,
                    src: self.src(),
                    bad_bit: body.as_miette_span(),
                    decl: self.span(funclet.ret().unwrap().span()),
                },
            ));
        }

        let body_ir = self.ir.drain(prev_len..).collect::<Vec<_>>();

        self.scopes.pop();

        self.ir.push(IR::FuncLet {
            name: funclet.name().to_string(),
            args: funclet
                .args_raw()
                .as_ref()
                .map(|arg| {
                    arg.node
                        .iter()
                        .map(|a| (a.name.to_string(), a.ty.clone().into()))
                        .collect()
                })
                .unwrap_or_default(),
            internal: false,
            ret: expected.clone(),
            body: body_ir,
        });

        Ok(expected)
    }

    fn check_grouping(&mut self, group: &'a Grouping<'a>) -> Result<Type, TypeCheckError> {
        self.scopes.push();

        let prev_len = self.ir.len();

        let redirect_ir = if let Some(expr) = group.redirect() {
            let prev_len_redirect = self.ir.len();
            let got = self.inner(expr)?;

            if !matches!(got, Type::Stream) {
                return Err(TypeCheckError::VerifyError(
                    pass_one::VerifyError::MismatchedType {
                        expected: Type::Stream,
                        got,
                        src: self.src(),
                        bad_bit: expr.as_miette_span(),
                    },
                ));
            }

            let mut drained = self.ir.drain(prev_len_redirect..).collect::<Vec<_>>();
            let ir = drained
                .pop()
                .expect("redirect produced no IR but we found one above?");

            let stream_name = "STREAM";
            let unique = self.var_gen.fresh(stream_name);

            self.scopes
                .insert(stream_name, expr.span(), Type::Stream, &unique);

            Some((Box::new(ir), unique))
        } else {
            None
        };

        let stmts = group.stmts_raw();

        #[allow(unused_assignments) /* I KNOW CLIPPY OMFG */]
        let mut last_val_ty = Type::Unit;
        let mut last_expr_end = None;

        match stmts {
            [] => {
                last_val_ty = Type::Unit;
            }
            [rest @ .., last] => {
                #[allow(unused_assignments) /* I'm gonna shoot you, Clippy */]
                for stmt in rest {
                    last_val_ty = self.check_node(stmt)?;
                }

                let before = self.ir.len();
                last_val_ty = self.check_node(last)?;

                let mut emitted = self.ir.drain(before..).collect::<Vec<_>>();

                last_expr_end = emitted.pop();
            }
        }

        let inner_ir = self.ir.drain(prev_len..).collect::<Vec<_>>();

        self.scopes.pop();

        self.ir.push(IR::Grouping {
            inner: inner_ir,
            expr_end: last_expr_end.map(Box::new),
            redirect: redirect_ir,
        });

        Ok(last_val_ty)
    }

    fn check_funccall(
        &mut self,
        func: &'a FuncCall<'a>,
        span: pest::Span<'a>,
    ) -> Result<Type, TypeCheckError> {
        let def =
            self.funcs
                .lookup(func.name())
                .ok_or_else(|| TypeCheckError::UnknownFunction {
                    name: func.name().to_string(),
                    src: self.src(),
                    bad_bit: self.span(span),
                })?;

        let args = func.args_raw().as_ref().map(|a| &a.node[..]).unwrap_or(&[]);

        let got = args.len();
        let expected = def.args.len();

        if got != expected {
            return Err(TypeCheckError::VerifyError(
                pass_one::VerifyError::ArgumentLengthMismatch {
                    expected,
                    got,
                    src: self.src(),
                    bad_bit: self.span(span),
                },
            ));
        }

        let mut args_ir = Vec::with_capacity(args.len());

        for (slot, (arg_expr, expected)) in args.iter().zip(&def.args).enumerate() {
            let prev_len = self.ir.len();

            let got = self.check_node(arg_expr)?;

            let arg_ir = self.ir.drain(prev_len..).collect::<Vec<_>>();
            args_ir.extend(arg_ir);

            let expected = expected.clone();

            if got != expected {
                return Err(TypeCheckError::VerifyError(
                    pass_one::VerifyError::ArgumentTypeMismatch {
                        slot,
                        expected,
                        got,
                        src: self.src(),
                        bad_bit: arg_expr.as_miette_span(),
                    },
                ));
            }
        }

        let mut ret_ty = def.ret.clone();

        let unwrap = func.has_unwrap();

        if let Some(unwrap) = func.get_unwrap() {
            match ret_ty {
                Type::Result(ok, _) => {
                    ret_ty = *ok;
                }
                _ => {
                    return Err(TypeCheckError::VerifyError(
                        pass_one::VerifyError::UnwrapNonResult {
                            src: self.src(),
                            bad_bit: unwrap.as_miette_span(),
                        },
                    ));
                }
            }
        }

        self.ir.push(IR::FuncCall {
            name: func.name().to_string(),
            args: args_ir,
            unwrap,
        });

        Ok(ret_ty)
    }

    fn check_atomic(&mut self, atom: &'a PAtomic<'a>) -> Result<Type, TypeCheckError> {
        match atom {
            PAtomic::Integer(i) => {
                self.ir.push(IR::Integer(**i));
                Ok(Type::Integer)
            }
            PAtomic::String(s) => {
                self.ir.push(IR::String(s.to_string()));
                Ok(Type::String)
            }
            PAtomic::Array(spanned) => {
                let elems = &spanned.node.node;

                match elems.iter().as_slice() {
                    [] => Err(TypeCheckError::VerifyError(
                        pass_one::VerifyError::EmptyArrayInfer,
                    )),
                    [first, rest @ ..] => {
                        let prev_len = self.ir.len();

                        let expected = self.check_node(first)?;

                        for elem in rest {
                            let got = self.check_node(elem)?;

                            if got != expected {
                                return Err(TypeCheckError::VerifyError(
                                    pass_one::VerifyError::MismatchedArrayElements {
                                        expected,
                                        got,
                                        src: self.src(),
                                        bad_bit: elem.as_miette_span(),
                                    },
                                ));
                            }
                        }

                        let elems_ir = self.ir.drain(prev_len..).collect::<Vec<_>>();

                        self.ir.push(IR::Array(elems_ir));

                        Ok(Type::Array(Box::new(expected)))
                    }
                }
            }
            PAtomic::Ident(ident) => {
                let name = ident.node;

                let (ty, unique) = self.scopes.get_unique_ident(name).ok_or_else(|| {
                    TypeCheckError::UnknownVariable {
                        name: name.to_string(),
                        src: self.src(),
                        bad_bit: self.span(atom.span()),
                    }
                })?;

                self.ir.push(IR::Ident(unique));

                Ok(ty)
            }
            PAtomic::Unit(_) => {
                self.ir.push(IR::Unit);
                Ok(Type::Unit)
            }
            PAtomic::Result(spanned) => {
                let prev_len = self.ir.len();

                let ok_ty = self.check_atomic(&spanned.0)?;
                let err_ty = self.check_atomic(&spanned.1)?;

                let mut parts = self.ir.drain(prev_len..).collect::<Vec<_>>();

                let err_ir = parts.pop().expect("missing err IR");
                let ok_ir = parts.pop().expect("missing ok IR");

                self.ir.push(IR::Result {
                    ok: Box::new(ok_ir),
                    err: Box::new(err_ir),
                });

                Ok(Type::Result(Box::new(ok_ty), Box::new(err_ty)))
            }
        }
    }
}
