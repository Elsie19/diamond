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

    fn func_call_args(input: Node) -> DResult<BPArr> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [expr(single)] => Spanned::new(Box::new([single]), span),
            [expr(single), expr(rest)..] => {
                let mut v = vec![];
                v.push(single);
                v.extend(rest);
                Spanned::new(v.into_boxed_slice(), span)
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
mod test {
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
                    *name.node.atomic_unchecked().node.ident_unchecked(),
                    *alias.node.atomic_unchecked().node.ident_unchecked(),
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
            DIParser::parse(Rule::func_expr, string).expect("failed to parse alias expression");
        let input = inputs.single().expect("expected only one root node");
        let func = DIParser::func_expr(input).expect("failed to parse `alias_expr`");

        let (name, args, unwrap) = match func.into_inner() {
            PVal::FuncCall { name, args, unwrap } => (name, args, unwrap),
            _ => unreachable!("not `PVal::FuncCall`"),
        };

        let name = unsafe { *name.node.atomic_unchecked().node.ident_unchecked() };

        assert_eq!(name, "super_func");

        assert!(!unwrap);

        let Some(args) = args else {
            unreachable!("args are not empty!");
        };

        // ident, array, func_call
        assert_eq!(args.len(), 3);
    }
}
