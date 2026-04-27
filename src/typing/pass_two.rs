use std::rc::Rc;

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
    attrs: Attributes,
    ir: Vec<IR>,
}

#[derive(Debug)]
struct TypeAndIR {
    ty: Type,
    ir: IR,
}

#[derive(Debug, Clone, Copy)]
struct Attributes {
    array_homogeneity_required: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            array_homogeneity_required: true,
        }
    }
}

impl Attributes {
    pub fn homo_arrays_allowed(self) -> bool {
        self.array_homogeneity_required
    }

    pub fn hetero_arrays_allowed(self) -> bool {
        !self.homo_arrays_allowed()
    }
}

impl TypeAndIR {
    fn new(ty: Type, ir: IR) -> Self {
        Self { ty, ir }
    }

    fn ty(&self) -> &Type {
        &self.ty
    }

    fn ir(&self) -> &IR {
        &self.ir
    }

    fn into_ty(self) -> Type {
        self.ty
    }

    fn into_ir(self) -> IR {
        self.ir
    }
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

        #[label("used here")]
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
            attrs: Attributes::default(),
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

    pub fn check_program(&mut self, program: &'a UntypedAst<'a>) -> Result<(), TypeCheckError> {
        // We don't know exactly how much it'll generate, but at least one per AST branch is a good
        // guess.
        let mut ir = Vec::with_capacity(program.len());

        for node in program {
            let result = self.check_node(node)?;
            ir.push(result.into_ir());
        }

        self.ir = ir;

        Ok(())
    }

    fn inner(&mut self, sp: &'a Spanned<'a, Box<PVal<'a>>>) -> Result<TypeAndIR, TypeCheckError> {
        self.check_inner(&sp.node, sp.span())
    }

    fn check_node(&mut self, node: &'a SpannedPVal<'a>) -> Result<TypeAndIR, TypeCheckError> {
        self.check_inner(&node.node, node.span())
    }

    fn get_span_of_ident(&self, val: &PVal) -> Option<SourceSpan> {
        if let PVal::Atomic(spanned) = val
            && let PAtomic::Ident(name) = &**spanned
        {
            self.scopes.get_span(name.node).map(|span| self.span(*span))
        } else {
            None
        }
    }

    fn check_inner(
        &mut self,
        node: &'a PVal<'a>,
        span: pest::Span<'a>,
    ) -> Result<TypeAndIR, TypeCheckError> {
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
                let ir_and_val = self.check_inner(&spanned.node, spanned.span())?;

                Ok(TypeAndIR {
                    ty: Type::Unit,
                    ir: IR::Stmt(Rc::new(ir_and_val.into_ir())),
                })
            }
        }
    }

    fn check_for(&mut self, for_: &'a For<'a>) -> Result<TypeAndIR, TypeCheckError> {
        let loop_expr = for_.loop_raw().expr_raw();

        let iter_res = self.inner(loop_expr)?;

        let elem_ty = if let Type::Array(inner) = iter_res.ty {
            *inner
        } else {
            // We can get a little clever here. If what's trying to be used as an
            // iterable is not a constant, but an identifier, we can go find its span
            // and have even nicer error messages.
            let defined_here = self.get_span_of_ident(loop_expr);

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

        self.scopes
            .insert(bind_name.node, bind_name.span(), elem_ty, unique.clone());

        let body_res = self.inner(for_.body_raw())?;

        self.scopes.pop();

        Ok(TypeAndIR {
            ty: body_res.ty,
            ir: IR::For {
                bind: unique,
                iter: Rc::new(iter_res.ir),
                body: Rc::new(body_res.ir),
            },
        })
    }

    fn check_match(&mut self, match_: &'a Match<'a>) -> Result<TypeAndIR, TypeCheckError> {
        let expr_raw = match_.expr_raw();

        let expr_res = self.inner(expr_raw)?;

        let (ok_ty, err_ty) = match expr_res.ty {
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

        let arms = match_.arms_raw();

        let mut arms_ir = Vec::with_capacity(arms.len());

        for arm in arms {
            self.scopes.push();

            let (bind_ty, is_ok) = match &arm.res {
                PMatchCase::Ok(_) => (ok_ty.clone(), true),
                PMatchCase::Err(_) => (err_ty.clone(), false),
            };

            let unique = self.var_gen.fresh(arm.res.name());

            self.scopes
                .insert(&arm.inner, arm.inner.span(), bind_ty, &*unique);

            let arm_res = self.inner(&arm.expr)?;

            self.scopes.pop();

            arms_ir.push(IRMatchArm {
                bind: unique,
                is_ok,
                body: Box::new(arm_res.ir.clone()),
            });

            if let Some(got) = &result_ty {
                if *got != arm_res.ty {
                    let got = got.clone();
                    return Err(TypeCheckError::VerifyError(
                                pass_one::VerifyError::MismatchedMatchArms {
                                    expected: arm_res.ty.clone(),
                                    got,
                                    src: self.src(),
                                    cur_branch: self.span(arm.expr.span()),
                                    prev_branch: self.span(last_span.expect("how are we failing on a current branch if we don't have a previous")),
                                },
                            ));
                }
            } else {
                last_span = Some(arm.expr.span());
                result_ty = Some(arm_res.into_ty());
            }
        }

        Ok(TypeAndIR {
            ty: result_ty.unwrap_or_default(),
            ir: IR::Match {
                expr: Rc::new(expr_res.ir),
                arms: arms_ir,
            },
        })
    }

    fn check_let(&mut self, let_: &'a Let<'a>) -> Result<TypeAndIR, TypeCheckError> {
        let expr_res = self.inner(let_.expr_raw())?;

        let name = let_.name_raw();
        let unique = self.var_gen.fresh(**name);

        self.scopes
            .insert(name, name.span(), expr_res.ty.clone(), &*unique);

        Ok(TypeAndIR {
            ty: expr_res.ty.clone(),
            ir: IR::Let {
                name: unique,
                ty: expr_res.ty,
                value: Rc::new(expr_res.ir),
            },
        })
    }

    fn check_funclet(&mut self, funclet: &'a FuncLet<'a>) -> Result<TypeAndIR, TypeCheckError> {
        let expected = funclet.ret().cloned().map_or(Type::Unit, Type::from);

        if funclet.is_internal() {
            return Ok(TypeAndIR {
                ty: expected,
                ir: IR::Unit,
            });
        }

        self.scopes.push();

        let args = funclet.args_raw();

        let mut lowered_args = Vec::with_capacity(match args {
            Some(o) => o.len(),
            None => 0,
        });

        if let Some(args) = args {
            for arg in &args.node {
                let unique = self.var_gen.fresh(*arg.name);

                self.scopes
                    .insert(&arg.name, arg.name.span(), arg.ty.clone().into(), &*unique);

                lowered_args.push((unique, arg.ty.clone().into()));
            }
        }

        let body = funclet.body_raw();
        let got = self.inner(body)?;

        if *got.ty() != expected {
            self.scopes.pop();
            return Err(TypeCheckError::VerifyError(
                pass_one::VerifyError::InvalidReturnType {
                    expected,
                    got: got.ty,
                    src: self.src(),
                    bad_bit: body.as_miette_span(),
                    decl: self.span(funclet.ret().unwrap().span()),
                },
            ));
        }

        self.scopes.pop();

        Ok(TypeAndIR {
            ty: expected.clone(),
            ir: IR::FuncLet {
                name: funclet.name().into(),
                args: lowered_args,
                internal: false,
                ret: expected,
                body: Rc::new(got.ir),
            },
        })
    }

    fn check_grouping(&mut self, group: &'a Grouping<'a>) -> Result<TypeAndIR, TypeCheckError> {
        self.scopes.push();

        let mut inner_ir = vec![];
        let mut last_val_ty = Type::Unit;

        let redirect_ir = if let Some(expr) = group.redirect() {
            let got = self.inner(expr)?;

            if *got.ty() != Type::Stream {
                return Err(TypeCheckError::VerifyError(
                    pass_one::VerifyError::MismatchedType {
                        expected: Type::Stream,
                        got: got.into_ty(),
                        src: self.src(),
                        bad_bit: expr.as_miette_span(),
                    },
                ));
            }

            let stream_name = "STREAM";
            let unique = self.var_gen.fresh(stream_name);

            self.scopes
                .insert(stream_name, expr.span(), Type::Stream, &*unique);

            Some((Box::new(got.ir.clone()), unique))
        } else {
            None
        };

        for stmt in group.stmts_raw() {
            let res = self.check_node(stmt)?;
            last_val_ty = res.ty;
            inner_ir.push(res.ir);
        }

        self.scopes.pop();

        Ok(TypeAndIR {
            ty: last_val_ty,
            ir: IR::Grouping {
                inner: inner_ir,
                redirect: redirect_ir,
            },
        })
    }

    fn check_funccall(
        &mut self,
        func: &'a FuncCall<'a>,
        span: pest::Span<'a>,
    ) -> Result<TypeAndIR, TypeCheckError> {
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
            let expected = expected.clone();

            let old_homo = self.attrs.array_homogeneity_required;
            if expected.is_any_array() {
                self.attrs.array_homogeneity_required = false;
            }

            let got = self.check_node(arg_expr)?;

            self.attrs.array_homogeneity_required = old_homo;

            if !expected.is_any() && *got.ty() != expected {
                let defined_here = self.get_span_of_ident(arg_expr);
                return Err(TypeCheckError::VerifyError(
                    pass_one::VerifyError::ArgumentTypeMismatch {
                        slot,
                        expected,
                        got: got.into_ty(),
                        src: self.src(),
                        bad_bit: arg_expr.as_miette_span(),
                        defined_here,
                    },
                ));
            }

            args_ir.push(got.ir);
        }

        let mut ret_ty = def.ret.clone();

        let unwrap = func.has_unwrap();

        if let Some(unwrap_span) = func.get_unwrap() {
            match ret_ty {
                Type::Result(ok, _) => {
                    ret_ty = *ok;
                }
                _ => {
                    return Err(TypeCheckError::VerifyError(
                        pass_one::VerifyError::UnwrapNonResult {
                            src: self.src(),
                            bad_bit: unwrap_span.as_miette_span(),
                        },
                    ));
                }
            }
        }

        Ok(TypeAndIR {
            ty: ret_ty,
            ir: IR::FuncCall {
                name: func.name().into(),
                args: args_ir,
                unwrap,
            },
        })
    }

    fn check_atomic(&mut self, atom: &'a PAtomic<'a>) -> Result<TypeAndIR, TypeCheckError> {
        match atom {
            PAtomic::Integer(i) => Ok(TypeAndIR {
                ty: Type::Integer,
                ir: IR::Integer(**i),
            }),
            PAtomic::String(s) => {
                let str = **s;
                Ok(TypeAndIR {
                    ty: Type::String,
                    ir: IR::String(str.into()),
                })
            }
            PAtomic::Array(spanned) => {
                let elems = &spanned.node.node;

                match elems.iter().as_slice() {
                    [] => Ok(TypeAndIR {
                        ty: Type::Array(Box::new(Type::Any)),
                        ir: IR::Array(Vec::with_capacity(0)),
                    }),
                    [first, rest @ ..] => {
                        let heterorrays = self.attrs.hetero_arrays_allowed();

                        let first = self.check_node(first)?;

                        let mut ir = Vec::with_capacity(1 + rest.len());
                        ir.push(first.ir);

                        let expected = first.ty;

                        for elem in rest {
                            let got = self.check_node(elem)?;

                            if !heterorrays && *got.ty() != expected {
                                return Err(TypeCheckError::VerifyError(
                                    pass_one::VerifyError::MismatchedArrayElements {
                                        expected,
                                        got: got.into_ty(),
                                        src: self.src(),
                                        bad_bit: elem.as_miette_span(),
                                    },
                                ));
                            }

                            ir.push(got.ir);
                        }

                        Ok(TypeAndIR {
                            ty: Type::Array(Box::new(expected)),
                            ir: IR::Array(ir),
                        })
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

                Ok(TypeAndIR {
                    ty,
                    ir: IR::Ident(unique),
                })
            }
            PAtomic::Unit(_) => Ok(TypeAndIR {
                ty: Type::Unit,
                ir: IR::Unit,
            }),
            PAtomic::Result(spanned) => {
                let ok_res = self.check_atomic(&spanned.0)?;
                let err_res = self.check_atomic(&spanned.1)?;

                Ok(TypeAndIR {
                    ty: Type::Result(Box::new(ok_res.ty), Box::new(err_res.ty)),
                    ir: IR::Result {
                        ok: Box::new(ok_res.ir.clone()),
                        err: Box::new(err_res.ir.clone()),
                    },
                })
            }
        }
    }
}
