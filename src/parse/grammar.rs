use pest_consume::{Error, Parser, match_nodes};

use super::types::*;

type DResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "src/parse/grammar.pest"]
struct DIParser;

#[pest_consume::parser]
impl DIParser {
    fn EOI(_input: Node) -> DResult<()> {
        Ok(())
    }

    // EXPRESSIONS //

    fn expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [alias_expr(expr)] => expr,
            [func_expr(expr)] => expr,
            [value(expr)] => Spanned::new(expr, span),
        ))
    }

    fn alias_expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [func_sigil_and_name(name), func_sigil_and_name(alias)] =>
                Spanned::new(PVal::Alias {
                    name: name.into_boxed(),
                    alias: alias.into_boxed()
                }, span)
        ))
    }

    fn func_expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [func_sigil_and_name(name), func_call_args(args), result_unwrap(unwrap)] =>
                Spanned::new(PVal::FuncCall { name: name.into_boxed(), args: Some(args), unwrap }, span),
            [func_sigil_and_name(name), func_call_args(args)] =>
                Spanned::new(PVal::FuncCall { name: name.into_boxed(), args: Some(args), unwrap: false }, span),
            [func_sigil_and_name(name), result_unwrap(unwrap)] =>
                Spanned::new(PVal::FuncCall { name: name.into_boxed(), args: None, unwrap }, span),
            [func_sigil_and_name(name)] =>
                Spanned::new(PVal::FuncCall { name: name.into_boxed(), args: None, unwrap: false }, span)
        ))
    }

    fn func_def_expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();

        Ok(match_nodes!(input.into_children();
            [func_sigil_and_name(name), expr(body)] => {
                Spanned::new(
                    PVal::FuncLet {
                        name: name.into_boxed(),
                        args: Spanned::new(Box::new([]), span),
                        body: body.into_boxed(),
                    },
                    span,
                )
            },
            [func_sigil_and_name(name), func_def_args(args), expr(body)] => {
                Spanned::new(
                    PVal::FuncLet {
                        name: name.into_boxed(),
                        args,
                        body: body.into_boxed(),
                    },
                    span,
                )
            }
        ))
    }

    fn func_call_args(input: Node) -> DResult<BPArr> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [expr(single)] => Spanned::new(Box::new([single]), span),
            [expr(single), expr(rest)..] => {
                let mut v = vec![single];
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
                let mut v = vec![first];
                v.extend(rest);
                Spanned::new(v.into_boxed_slice(), span)
            }
        ))
    }

    fn func_arg(input: Node) -> DResult<FuncArg> {
        Ok(match_nodes!(input.into_children();
            [ident(name), type_name(ty)] => {
                let name = unsafe { name.into_ident_unchecked() };
                FuncArg { name, ty }
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
        Ok(PAtomic::String(Spanned::new(
            input.as_str(),
            input.as_span(),
        )))
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
            [atomic_type(ty)] => ty,
        ))
    }

    fn type_array(input: Node) -> DResult<PType> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [type_name(arr)] => PType::Array(Spanned::new(Box::new(arr), span)),
        ))
    }

    fn atomic_type(input: Node) -> DResult<PType> {
        let span = input.as_span();
        match input.as_str() {
            txt @ "stream" => Ok(PType::Stream(Spanned::new(txt, span))),
            txt @ "string" => Ok(PType::String(Spanned::new(txt, span))),
            txt @ "file" => Ok(PType::File(Spanned::new(txt, span))),
            txt @ "()" => Ok(PType::Unit(Spanned::new(txt, span))),
            err => Err(input.error(err)),
        }
    }

    /*
     *  alias @foo = @bar;
     *        ^^^^ is for `func_sigil_and_name`
     *         ^^^ is for underlying ident
     *
     */
    fn func_sigil_and_name(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [ident(name)] =>
                Spanned::new(
                    PVal::Atomic(
                        Spanned::new( // ident
                            name.clone(),
                            name.span()
                        )
                    ),
                span) // sigil and ident
        ))
    }

    fn result_unwrap(input: Node) -> DResult<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod simple_parsing {
    use super::*;

    #[test]
    fn alias() {
        let string = "alias @foo = @bar";
        let inputs =
            DIParser::parse(Rule::alias_expr, string).expect("failed to parse alias expression");
        let input = inputs.single().expect("expected only one root node");
        let alias = DIParser::alias_expr(input).expect("failed to parse `alias_expr`");
        let (name, alias) = match alias.node {
            PVal::Alias { name, alias } => unsafe {
                (
                    *name
                        .node
                        .into_atomic_unchecked()
                        .node
                        .into_ident_unchecked(),
                    *alias
                        .node
                        .into_atomic_unchecked()
                        .node
                        .into_ident_unchecked(),
                )
            },
            _ => unreachable!("not `PVal::Alias`"),
        };

        assert_eq!(name, "foo");
        assert_eq!(alias, "bar");
    }

    #[test]
    fn func_call() {
        let string = r###"@super_func(ident, ["string", 0], @output())"###;
        let inputs =
            DIParser::parse(Rule::func_expr, string).expect("failed to parse func expression");
        let input = inputs.single().expect("expected only one root node");
        let func = DIParser::func_expr(input).expect("failed to parse `func_expr`");

        let (name, args, unwrap) = match func.into_inner() {
            PVal::FuncCall { name, args, unwrap } => (name, args, unwrap),
            _ => unreachable!("not `PVal::FuncCall`"),
        };

        let name = unsafe {
            *name
                .node
                .into_atomic_unchecked()
                .node
                .into_ident_unchecked()
        };

        assert_eq!(name, "super_func");

        assert!(!unwrap);

        let Some(args) = args else {
            unreachable!("args are not empty!");
        };

        // ident, array, func_call
        assert_eq!(args.len(), 3);
    }
}

#[cfg(test)]
mod complex_parsing {
    use super::*;

    #[test]
    fn func_def_to_another_func() {
        let string = "let @foo(x: string) = @bar(x)!";
        let inputs = DIParser::parse(Rule::func_def_expr, string)
            .expect("failed to parse func_def_expr expression");
        let input = inputs.single().expect("expected only one root node");
        let func = DIParser::func_def_expr(input).expect("failed to parse `func_def_expr`");

        let (name, args, _) = unsafe { func.node.into_func_let_unchecked() };
        let args = args.node;

        let name = unsafe {
            *name
                .node
                .into_atomic_unchecked()
                .node
                .into_ident_unchecked()
        };

        assert_eq!(name, "foo");

        assert_eq!(args.len(), 1);

        let arg = &args[0];

        assert_eq!(arg.name, "x");
    }

    #[test]
    fn func_def_to_type() {
        let string = "let @f() = ()";
        let inputs = DIParser::parse(Rule::func_def_expr, string)
            .expect("failed to parse func_def_expr expression");
        let input = inputs.single().expect("expected only one root node");
        let func = DIParser::func_def_expr(input).expect("failed to parse `func_def_expr`");

        let (name, args, body) = unsafe { func.node.into_func_let_unchecked() };
        let args = args.node;

        let name = unsafe {
            *name
                .node
                .into_atomic_unchecked()
                .node
                .into_ident_unchecked()
        };

        assert_eq!(name, "f");

        assert_eq!(args.len(), 0);

        assert!(matches!(
            unsafe { body.node.into_atomic_unchecked() }.node,
            PAtomic::Unit(_)
        ))
    }
}
