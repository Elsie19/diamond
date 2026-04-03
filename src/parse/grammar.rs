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

    pub fn alias_expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [func_sigil_and_name(name), func_sigil_and_name(alias)] =>
                Spanned::new(PVal::Alias {
                    name: name.into_boxed(),
                    alias: alias.into_boxed()
                }, span)
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
}
