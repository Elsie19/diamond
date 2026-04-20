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
        strata::web::Web,
        types::Type,
    },
};

#[derive(Debug)]
pub struct TypeChecker<'a> {
    funcs: &'a FuncTable<'a>,
    scopes: ScopeStack<'a>,
    source: NamedSource<String>,
    web: Web,
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
    pub fn new<T>(funcs: &'a FuncTable<'a>, file_name: &'a str, prog_text: &T) -> Self
    where
        T: ToString,
    {
        Self {
            funcs,
            scopes: ScopeStack::new(),
            source: NamedSource::new(file_name, prog_text.to_string()).with_language("diamond"),
            web: Web::new(),
        }
    }

    fn src(&self) -> NamedSource<String> {
        self.source.clone()
    }

    fn span(&self, span: pest::Span<'a>) -> SourceSpan {
        spest_to_smiette(span)
    }

    pub fn check_program(&mut self, program: &'a UntypedAst<'a>) -> Result<(), TypeCheckError> {
        for node in program {
            self.check_node(node)?;
        }

        Ok(())
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
                self.check_inner(&spanned.node, spanned.span())?;
                Ok(Type::Unit)
            }
        }
    }

    fn check_for(&mut self, for_: &'a For<'a>) -> Result<Type, TypeCheckError> {
        let loop_expr = for_.loop_raw().expr_raw();

        let iter_ty = self.inner(loop_expr)?;

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

        self.scopes.insert(
            for_.loop_raw().bind_raw(),
            for_.loop_raw().bind_raw().span(),
            elem_ty,
        );

        let for_ret_ty = self.inner(for_.body_raw())?;

        self.scopes.pop();

        Ok(for_ret_ty)
    }

    fn check_match(&mut self, match_: &'a Match<'a>) -> Result<Type, TypeCheckError> {
        let expr_raw = match_.expr_raw();

        let expr_ty = self.inner(expr_raw)?;

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

        for arm in match_.arms_raw() {
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

            let expected = self.inner(&arm.expr)?;

            self.scopes.pop();

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

        Ok(result_ty.unwrap_or_default())
    }

    fn check_let(&mut self, let_: &'a Let<'a>) -> Result<Type, TypeCheckError> {
        let ty = self.inner(let_.expr_raw())?;

        let name = let_.name_raw();

        self.scopes.insert(name, name.span(), ty.clone());
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
                self.scopes
                    .insert(&arg.name, arg.name.span(), arg.ty.clone().into());
            }
        }

        let result = {
            let body = funclet.body_raw();
            let got = self.inner(body)?;
            if got != expected {
                Err(TypeCheckError::VerifyError(
                    pass_one::VerifyError::InvalidReturnType {
                        expected,
                        got,
                        src: self.src(),
                        bad_bit: body.as_miette_span(),
                        decl: self.span(funclet.ret().unwrap().span()),
                    },
                ))
            } else {
                Ok(expected)
            }
        };

        self.scopes.pop();
        result
    }

    fn check_grouping(&mut self, group: &'a Grouping<'a>) -> Result<Type, TypeCheckError> {
        self.scopes.push();

        let stmts = group.stmts_raw();

        let mut last_val_ty = Type::Unit;

        for stmt in stmts {
            last_val_ty = self.check_node(stmt)?;
        }

        if let Some(expr) = group.redirect() {
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
        }

        self.scopes.pop();

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

        for (slot, (arg_expr, expected)) in args.iter().zip(&def.args).enumerate() {
            let got = self.check_node(arg_expr)?;
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

        Ok(ret_ty)
    }

    fn check_atomic(&mut self, atom: &'a PAtomic<'a>) -> Result<Type, TypeCheckError> {
        match atom {
            PAtomic::Integer(_) => {
                self.web.insert(Type::Integer);
                Ok(Type::Integer)
            }
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
                                        src: self.src(),
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
                        src: self.src(),
                        bad_bit: self.span(atom.span()),
                    })
            }
            PAtomic::Unit(_) => Ok(Type::Unit),
            PAtomic::Result(spanned) => Ok(Type::Result(
                Box::new(self.check_atomic(&spanned.0)?),
                Box::new(self.check_atomic(&spanned.1)?),
            )),
        }
    }
}
