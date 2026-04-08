use pest_consume::{Error, Parser, match_nodes};

use super::types::*;

pub type UntypedAst<'a> = Vec<SpannedPVal<'a>>;

type DResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "src/parse/grammar.pest"]
pub struct DIParser;

pub fn parse_di<'a>(input_str: &'a str, file: &'a str) -> Result<UntypedAst<'a>, ()> {
    let inputs = DIParser::parse(Rule::program, input_str).map_err(|e| {
        let e = e.with_path(file).renamed_rules(|rule| match *rule {
            Rule::EOI => "end of file".to_string(),
            Rule::stmt => "statement".to_string(),
            Rule::type_name => "type".to_string(),
            Rule::func_sigil_and_name => "function call".to_string(),
            Rule::func_def_expr => "function definition".to_string(),
            Rule::assign_expr => "assignment".to_string(),
            Rule::match_expr => "match".to_string(),
            Rule::for_expr => "for loop".to_string(),
            bla => format!("{:?}", bla),
        });

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
            [stmt(stmts).., EOI(())] => stmts.collect(),
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
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            // SAFETY: `ident` returns a [`PAtomic`] but underneath we know it's a string.
            [ident(name), expr(expr)] => Spanned::new(PVal::Let { name: unsafe { name.into_ident_unchecked() }, expr: expr.into_boxed() }, span),
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
                        ret: None,
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
                        ret: None,
                        body: body.into_boxed(),
                    },
                    span,
                )
            },
            [func_sigil_and_name(name), func_def_ret(ret), expr(body)] => {
                Spanned::new(
                    PVal::FuncLet {
                        name: name.into_boxed(),
                        args: Spanned::new(Box::new([]), span),
                        ret: Some(ret),
                        body: body.into_boxed(),
                    },
                    span,
                )
            },
            [func_sigil_and_name(name), func_def_args(args), func_def_ret(ret), expr(body)] => {
                Spanned::new(
                    PVal::FuncLet {
                        name: name.into_boxed(),
                        args,
                        ret: Some(ret),
                        body: body.into_boxed(),
                    },
                    span,
                )
            }
        ))
    }

    fn func_def_ret(input: Node) -> DResult<PType> {
        Ok(match_nodes!(input.into_children();
            [type_name(name)] => name,
        ))
    }

    fn grouping(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        let (stmts, return_expr, redirect) = match_nodes!(input.into_children();
            [stmt(stmts)..] => (stmts.collect(), None, None),
            [stmt(stmts).., expr(ret)] => (stmts.collect(), Some(ret.into_boxed()), None),
            [stmt(stmts).., redirect(redirect)] => (stmts.collect(), None, Some(redirect.into_boxed())),
            [stmt(stmts).., expr(ret), redirect(redirect)] => (stmts.collect(), Some(ret.into_boxed()), Some(redirect.into_boxed())),
        );

        Ok(Spanned::new(
            PVal::Grouping {
                stmts,
                return_expr,
                redirect,
            },
            span,
        ))
    }

    fn for_expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [for_inner(loop_), expr(body)] => Spanned::new(PVal::For { loop_, body: body.into_boxed() }, span),
        ))
    }

    fn for_inner(input: Node) -> DResult<Spanned<PForInner>> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [ident(bind), expr(expr)] => Spanned::new(PForInner { bind: unsafe { bind.into_ident_unchecked() }, expr: expr.into_boxed() }, span),
        ))
    }

    fn match_expr(input: Node) -> DResult<SpannedPVal> {
        let span = input.as_span();
        let arms = match_nodes!(input.into_children();
            [expr(expr), match_arm(arm), match_arm(rest)..] => PVal::Match { expr: expr.into_boxed(), arms: {
                let mut v = vec![arm];
                v.extend(rest);
                v.into_boxed_slice()
            } },
            [expr(expr), match_arm(arm)] => PVal::Match { expr: expr.into_boxed(), arms: Box::new([arm]) },
        );

        Ok(Spanned::new(arms, span))
    }

    fn match_arm(input: Node) -> DResult<Spanned<PMatchArm>> {
        let span = input.as_span();
        Ok(match_nodes!(input.into_children();
            [result_branch(res), ident(inner), expr(expr)] => Spanned::new(PMatchArm { res, inner: unsafe { inner.into_ident_unchecked() }, expr: expr.into_boxed() }, span)
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
                // SAFETY: We know that `ident` returns an [`ident`].
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
            [atomic_type(ty)] => ty,
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
            err => Err(input.error(err)),
        }
    }

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

    #[test]
    fn assign() {
        let string = "let a = 0";
        let inputs =
            DIParser::parse(Rule::assign_expr, string).expect("failed to parse assign expression");
        let input = inputs.single().expect("expected only one root node");
        let assign = DIParser::assign_expr(input).expect("failed to parse `assign_expr`");

        let (ident, expr) = unsafe { assign.node.into_let_unchecked() };

        let expr = unsafe {
            expr.node
                .into_atomic_unchecked()
                .node
                .into_integer_unchecked()
        };

        assert_eq!(ident, "a");
        assert_eq!(expr, 0);
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

        let (name, args, _, _) = unsafe { func.node.into_func_let_unchecked() };
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

        let (name, args, ret, body) = unsafe { func.node.into_func_let_unchecked() };
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

    #[test]
    fn func_def_to_grouping() {
        let string = r###"let @tee(file: stream, txt: [string]) = {
                            @printf("%s", txt);
                            @dump(file, txt);
        };"###;

        let inputs = DIParser::parse(Rule::func_def_expr, string)
            .expect("failed to parse func_def_expr expression");
        let input = inputs.single().expect("expected only one root node");
        let func = DIParser::func_def_expr(input).expect("failed to parse `func_def_expr`");

        let (name, args, ret, body) = unsafe { func.node.into_func_let_unchecked() };
        let args = args.node;

        let name = unsafe {
            *name
                .node
                .into_atomic_unchecked()
                .node
                .into_ident_unchecked()
        };

        assert_eq!(name, "tee");

        assert_eq!(args.len(), 2);

        assert!(matches!(*body.node, PVal::Grouping { .. }))
    }

    #[test]
    fn match_() {
        let string = r###"match (@nth(ARGV, 0)) {
                ok o = o,
                err e = @panic("expected file to be passed"),
             } # ty : file"###;

        let inputs = DIParser::parse(Rule::match_expr, string)
            .expect("failed to parse match_expr expression");
        let input = inputs.single().expect("expected only one root node");
        let match_ = DIParser::match_expr(input).expect("failed to parse `match_expr`");

        let (expr, arms) = unsafe { match_.node.into_match_unchecked() };

        assert!(matches!(**expr, PVal::FuncCall { .. }));

        assert_eq!(arms.len(), 2);

        let (ok, err) = (&arms[0], &arms[1]);

        assert!(ok.ok());
        assert!(err.err());

        let ok_expr = unsafe { ok.expr.node.as_atomic_unchecked().node.as_ident_unchecked() };
        let (func_name, func_args, func_unwrap) = unsafe { err.expr.node.as_func_call_unchecked() };

        let func_name = unsafe {
            &func_name
                .node
                .as_atomic_unchecked()
                .node
                .as_ident_unchecked()
        };

        let func_args = &func_args.as_ref().expect("argument count should be 1").node;
        assert_eq!(func_args.len(), 1);
        let func_arg = unsafe {
            func_args[0]
                .node
                .as_atomic_unchecked()
                .node
                .as_string_unchecked()
                .node
        };

        assert_eq!(*ok_expr, "o");
        assert_eq!(**func_name, "panic");
        assert_eq!(func_arg, "expected file to be passed");
        assert!(!func_unwrap);
    }

    #[test]
    fn for_() {
        let string = r###"for (a in @range(0, 50)) {
            @printf("%d\n", a);
        }"###;

        let inputs =
            DIParser::parse(Rule::for_expr, string).expect("failed to parse for_expr expression");
        let input = inputs.single().expect("expected only one root node");
        let for_ = DIParser::for_expr(input).expect("failed to parse `for_expr`");

        let (loop_, body) = unsafe { for_.as_for_unchecked() };

        let bind = &loop_.bind;

        assert_eq!(*bind, "a");

        assert!(matches!(***body, PVal::Grouping { .. }))
    }

    #[test]
    fn program() {
        let string = r###"# returns unit type.
let @tee(file: stream, txt: string) = {
    @printf("%s", txt);
    @dump(file, txt);
};

let @bar() = { 5 };

# set file to the first file inputted.
let file = @f(match (@nth(ARGV, 0)) {
                ok o = o,
                err e = @panic("expected file to be passed"),
             }); # ty : file
let output = @open(@create(@f("kvs.txt"))!)!; # ty : stream

{
    let first_line = @nth(STREAM, 0)!; # ty : string
    let csv_header_split = @split(first_line, ","); # ty : [string]
    let csv_length = @length(csv_header_split); # ty : integer
    let header = @sprintf("%s\n", @join_str(csv_header_split, ",")); # ty : string
    @tee(output, header);

    # main loop.
    for (line in @skip(STREAM, 0)) {
        let line_split = @split(line, ",");
        @assert_eq(@length(line_split), csv_length);
        let txt = @sprintf("%s\n", @join_str(line_split, ","));
        @tee(output, txt);
    }
} < @open(file)!;"###;

        let inputs = DIParser::parse(Rule::program, string).expect("failed to parse program");
        let input = inputs.single().expect("expected only one root node");
        let program = DIParser::program(input).expect("failed to parse `program`");

        dbg!(program);
    }
}
