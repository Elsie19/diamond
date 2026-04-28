use pest_consume::{Error, Parser, match_nodes};

use crate::parse::types::{
    for_::PForInner,
    funclet::FuncArg,
    match_::{PMatchArm, PMatchCase},
};

use super::types::{BPArr, PAtomic, PType, PVal, Spanned, SpannedPVal};

pub trait MietteSpan {
    fn as_miette_span(&self) -> miette::SourceSpan;
}

pub type UntypedAst<'a> = Vec<SpannedPVal<'a>>;

type DResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "src/parse/grammar.pest"]
pub struct DIParser;

pub fn parse_di<'a>(input_str: &'a str, file: &'a str) -> Result<UntypedAst<'a>, ()> {
    let inputs = DIParser::parse(Rule::program, input_str).map_err(|e| {
        let e = e.with_path(file);

        eprintln!("{}", e);
    })?;

    let input = inputs.single().map_err(|e| {
        eprintln!("{e}");
    })?;

    DIParser::program(input).map_err(|e| {
        eprintln!("{e}");
    })
}

#[pest_consume::parser]
impl DIParser {
    fn EOI(_input: Node) -> DResult<()> {
        Ok(())
    }

    fn program(input: Node) -> DResult<Vec<SpannedPVal>> {
        Ok(match_nodes!(input.into_children();
            [stmt_or_expr(stmts).., EOI(())] => stmts.collect(),
        ))
    }

    fn stmt(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [expr(expr)] => Spanned::new(PVal::Stmt(expr.into_boxed()), span),
        ))
    }

    // EXPRESSIONS //

    fn expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [func_expr(expr)] => expr,
            [func_def_expr(expr)] => expr,
            [assign_expr(expr)] => expr,
            [grouping(expr)] => expr,
            [match_expr(expr)] => expr,
            [for_expr(expr)] => expr,
            [value(expr)] => Spanned::new(expr, span),
        ))
    }

    fn assign_expr(input: Node) -> DResult<SpannedPVal> {
        use crate::parse::types::let_::Let;

        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [ident(name),
             expr(expr)] => {
                Spanned::new(
                    PVal::Let(
                        Let::builder()
                            .name(name)
                            .expr(expr)
                            .build()),
                span)
            }
        ))
    }

    fn func_expr(input: Node) -> DResult<SpannedPVal> {
        use crate::parse::types::funccall::FuncCall;

        let span = input.as_span();

        let func = match_nodes!(input.into_children();
            [func_sigil_and_name(name), func_call_args(args), result_unwrap(unwrap)] =>
                FuncCall::builder()
                    .name(name)
                    .args(args)
                    .unwrap(unwrap)
                    .build(),
            [func_sigil_and_name(name), func_call_args(args)] =>
                FuncCall::builder()
                    .name(name)
                    .args(args)
                    .build(),
            [func_sigil_and_name(name), result_unwrap(unwrap)] =>
                FuncCall::builder()
                    .name(name)
                    .unwrap(unwrap)
                    .build(),
            [func_sigil_and_name(name)] =>
                FuncCall::builder()
                    .name(name)
                    .build(),
        );

        Ok(Spanned::new(PVal::FuncCall(func), span))
    }

    fn func_def_expr(input: Node) -> DResult<SpannedPVal> {
        use crate::parse::types::funclet::FuncLet;

        let span = input.as_span();

        let funclet = match_nodes!(input.into_children();
            [func_sigil_and_name(name),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .body(body)
                    .build()
            },
            [func_sigil_and_name(name),
             func_def_args(args),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .args(args)
                    .body(body)
                    .build()
            },
            [func_sigil_and_name(name),
             func_def_ret(ret),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .ret(ret)
                    .body(body)
                    .build()
            },
            [func_sigil_and_name(name),
             func_def_args(args),
             func_def_ret(ret),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .args(args)
                    .ret(ret)
                    .body(body)
                    .build()
            },
            [internal(internal),
             func_sigil_and_name(name),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .body(body)
                    .internal(internal)
                    .build()
            },
            [internal(internal),
             func_sigil_and_name(name),
             func_def_args(args),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .args(args)
                    .body(body)
                    .internal(internal)
                    .build()
            },
            [internal(internal),
             func_sigil_and_name(name),
             func_def_ret(ret),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .ret(ret)
                    .body(body)
                    .internal(internal)
                    .build()
            },
            [internal(internal),
             func_sigil_and_name(name),
             func_def_args(args),
             func_def_ret(ret),
             expr(body)] => {
                FuncLet::builder()
                    .name(name)
                    .args(args)
                    .ret(ret)
                    .body(body)
                    .internal(internal)
                    .build()
            }
        );

        Ok(Spanned::new(PVal::FuncLet(funclet), span))
    }

    fn internal(input: Node) -> DResult<bool> {
        Ok(true)
    }

    fn func_def_ret(input: Node) -> DResult<PType> {
        Ok(match_nodes!(input.into_children();
            [type_name(name)] => name,
        ))
    }

    fn grouping(input: Node) -> DResult<SpannedPVal> {
        use crate::parse::types::grouping::Grouping;

        let span = input.as_span();

        let group = match_nodes!(input.into_children();
            [stmt_or_expr(stmts)..] => {
                Grouping::builder()
                    .stmts(stmts.collect())
                    .build()
            },
            [stmt_or_expr(stmts)..,
             redirect(redirect)] => {
                Grouping::builder()
                    .stmts(stmts.collect())
                    .redirect(redirect.into_boxed())
                    .build()
            }
        );

        Ok(Spanned::new(PVal::Grouping(group), span))
    }

    fn stmt_or_expr(input: Node) -> DResult<SpannedPVal> {
        Ok(match_nodes!(input.into_children();
            [stmt(stmt)] => stmt,
            [expr(expr)] => expr,
        ))
    }

    fn for_expr(input: Node) -> DResult<SpannedPVal> {
        use crate::parse::types::for_::For;

        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [for_inner(loop_), expr(body)] => {
                Spanned::new(PVal::For(
                    For::builder()
                        .loop_(loop_)
                        .body(body)
                        .build()),
                span)
            }
        ))
    }

    fn for_inner(input: Node) -> DResult<Spanned<PForInner>> {
        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [ident(bind), expr(expr)] => {
                Spanned::new(
                    PForInner::builder()
                        .bind(bind)
                        .expr(expr)
                        .build(),
                span)
            }
        ))
    }

    fn match_expr(input: Node) -> DResult<SpannedPVal> {
        use crate::parse::types::match_::Match;

        let span = input.as_span();

        let match_ = match_nodes!(input.into_children();
            [expr(expr),
             match_arm(arm),
             match_arm(rest)..] => {
                let mut v = Vec::with_capacity(rest.size_hint().0 + 1);
                v.push(arm);
                v.extend(rest);

                Match::builder()
                    .expr(expr)
                    .arms(v.into_boxed_slice())
                    .build()
            },
            [expr(expr),
             match_arm(arm)] => {
                Match::builder()
                    .expr(expr)
                    .arms(Box::new([arm]))
                    .build()
            }
        );

        Ok(Spanned::new(PVal::Match(match_), span))
    }

    fn match_arm(input: Node) -> DResult<Spanned<PMatchArm>> {
        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [result_branch(res), ident(inner), expr(expr)] => {
                Spanned::new(
                    PMatchArm::builder()
                        .res(res)
                        .inner(inner)
                        .expr(expr)
                        .build(),
                span)
            }
        ))
    }

    fn result_branch(input: Node) -> DResult<PMatchCase> {
        let span = input.as_span();
        match input.as_str() {
            txt @ "ok" => Ok(PMatchCase::Ok(Spanned::new(txt, span))),
            txt @ "err" => Ok(PMatchCase::Err(Spanned::new(txt, span))),
            _ => unreachable!("add result_branch new fields"),
        }
    }

    fn func_call_args(input: Node) -> DResult<BPArr> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [expr(single)] => Spanned::new(Box::new([single]), span),
            [expr(single), expr(rest)..] => {
                let mut v = Vec::with_capacity(rest.size_hint().0 + 1);
                v.push(single);
                v.extend(rest);
                Spanned::new(v.into_boxed_slice(), span)
            }
        ))
    }

    fn func_def_args(input: Node) -> DResult<Spanned<Box<[FuncArg]>>> {
        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [func_arg(single)] => {
                Spanned::new(Box::new([single]), span)
            },
            [func_arg(first), func_arg(rest)..] => {
                let mut v = Vec::with_capacity(rest.size_hint().0 + 1);
                v.push(first);
                v.extend(rest);
                Spanned::new(v.into_boxed_slice(), span)
            }
        ))
    }

    fn func_arg(input: Node) -> DResult<FuncArg> {
        Ok(match_nodes!(input.into_children();
            [ident(name), type_name(ty)] => {
                FuncArg::builder()
                    .name(name)
                    .ty(ty)
                    .build()
            }
        ))
    }

    fn value(input: Node) -> DResult<PVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [ident(ident)] => PVal::Atomic(Spanned::new(ident, span)),
            [integer(integer)] => PVal::Atomic(Spanned::new(integer, span)),
            [string_lit(string)] => PVal::Atomic(Spanned::new(string, span)),
            [array_lit(array)] => PVal::Atomic(Spanned::new(array, span)),
            [unit_lit(unit)] => PVal::Atomic(Spanned::new(unit, span)),
        ))
    }

    // ATOMIC VALUES //
    fn integer(input: Node) -> DResult<PAtomic> {
        Ok(PAtomic::Integer(Spanned::new(
            input
                .as_str()
                .parse::<usize>()
                .map_err(|e| input.error(e))?,
            input.as_span(),
        )))
    }

    fn ident(input: Node) -> DResult<PAtomic> {
        Ok(PAtomic::Ident(Spanned::new(
            input.as_str(),
            input.as_span(),
        )))
    }

    fn string_lit(input: Node) -> DResult<PAtomic> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [inner_string(str)] => PAtomic::String(Spanned::new(str, span)),
        ))
    }

    fn inner_string(input: Node<'_>) -> DResult<&str> {
        Ok(input.as_str())
    }

    fn array_lit(input: Node) -> DResult<PAtomic> {
        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [] => {
                PAtomic::Array(Spanned::new(
                    Spanned::new(Box::new([]), span),
                    span
                ))
            },
            [func_call_args(args)] => PAtomic::Array(Spanned::new(args, span)),
        ))
    }

    fn unit_lit(input: Node) -> DResult<PAtomic> {
        Ok(PAtomic::Unit(Spanned::new(input.as_str(), input.as_span())))
    }

    fn type_name(input: Node) -> DResult<PType> {
        Ok(match_nodes!(input.into_children();
            [type_array(arr)] => arr,
            [result_type(res)] => res,
            [atomic_type(ty)] => ty,
        ))
    }

    fn result_type(input: Node) -> DResult<PType> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [type_name(ok), type_name(err)] => PType::Result(Spanned::new((Box::new(ok), Box::new(err)), span)),
        ))
    }

    fn type_array(input: Node) -> DResult<PType> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [type_name(arr)] => PType::Array(Spanned::new(Box::new(arr), span)),
        ))
    }

    fn redirect(input: Node) -> DResult<SpannedPVal> {
        Ok(match_nodes!(input.into_children();
            [expr(expr)] => expr,
        ))
    }

    fn atomic_type(input: Node) -> DResult<PType> {
        let span = input.as_span();
        match input.as_str() {
            txt @ "stream" => Ok(PType::Stream(Spanned::new(txt, span))),
            txt @ "string" => Ok(PType::String(Spanned::new(txt, span))),
            txt @ "file" => Ok(PType::File(Spanned::new(txt, span))),
            txt @ "unit" => Ok(PType::Unit(Spanned::new(txt, span))),
            txt @ "integer" => Ok(PType::Integer(Spanned::new(txt, span))),
            txt @ "unret" => Ok(PType::Unret(Spanned::new(txt, span))),
            txt @ "any" => Ok(PType::Any(Spanned::new(txt, span))),
            err => Err(input.error(err)),
        }
    }

    fn func_sigil_and_name(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [ident(name)] => {
                let name_span = name.span();
                Spanned::new(
                    PVal::Atomic(
                        Spanned::new(
                            name,
                            name_span
                        )
                    ),
                span)
            }
        ))
    }

    fn result_unwrap(input: Node) -> DResult<Spanned<bool>> {
        Ok(Spanned::new(true, input.as_span()))
    }
}

pub fn spest_to_smiette(span: pest::Span<'_>) -> miette::SourceSpan {
    miette::SourceSpan::from(span.start()..span.end())
}
