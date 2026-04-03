use std::ops::Deref;

use enum_as_inner::EnumAsInner;

pub type BPVal<'a> = Spanned<'a, Box<PVal<'a>>>;
pub type BPArr<'a> = Spanned<'a, Box<[Spanned<'a, PVal<'a>>]>>;

pub type SpannedPVal<'a> = Spanned<'a, PVal<'a>>;
pub type SpannedStr<'a> = Spanned<'a, &'a str>;

#[derive(Debug, Clone)]
pub struct Spanned<'a, T> {
    pub node: T,
    pub span: pest::Span<'a>,
}

impl<T> Deref for Spanned<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> PartialEq<T> for Spanned<'_, T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.node == *other
    }
}

impl<'a, T> Spanned<'a, T> {
    pub fn new(node: T, span: pest::Span<'a>) -> Self {
        Self { node, span }
    }

    pub fn into_inner(self) -> T {
        self.node
    }

    pub fn into_boxed(self) -> Spanned<'a, Box<T>> {
        Spanned {
            node: Box::new(self.node),
            span: self.span,
        }
    }

    pub const fn span(&self) -> pest::Span<'a> {
        self.span
    }

    pub fn as_miette_span(&self) -> miette::SourceSpan {
        let start = self.span().start();
        let end = self.span().end();
        (start, end - start).into()
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum PVal<'a> {
    Atomic(Spanned<'a, PAtomic<'a>>),
    FuncCall {
        name: BPVal<'a>,
        args: Option<BPArr<'a>>,
        unwrap: bool,
    },
    FuncLet {
        name: BPVal<'a>,
        args: Spanned<'a, Box<[FuncArg<'a>]>>,
        body: BPVal<'a>,
    },
    Grouping {
        stmts: BPArr<'a>,
        return_expr: Option<BPVal<'a>>,
        redirect: Option<BPVal<'a>>,
    },
    Match {
        expr: BPVal<'a>,
        arms: (),
    },
    For {
        var: BPVal<'a>,
        iter: BPVal<'a>,
        body: BPVal<'a>,
        return_expr: Option<BPVal<'a>>,
    },
    Let {
        name: SpannedStr<'a>,
        expr: BPVal<'a>,
    },
    Alias {
        name: BPVal<'a>,
        alias: BPVal<'a>,
    },
    Expr(BPVal<'a>),
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum PAtomic<'a> {
    Integer(Spanned<'a, usize>),
    String(Spanned<'a, &'a str>),
    Array(Spanned<'a, BPArr<'a>>),
    Ident(Spanned<'a, &'a str>),
    Unit(Spanned<'a, &'a str>),
}

impl<'a> PAtomic<'a> {
    pub fn span(&self) -> pest::Span<'a> {
        match self {
            Self::Array(a) => a.span,
            Self::Ident(i) => i.span,
            Self::String(s) => s.span,
            Self::Integer(i) => i.span,
            Self::Unit(u) => u.span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PType<'a> {
    Array(Spanned<'a, Box<Self>>),
    Stream(Spanned<'a, &'a str>),
    String(Spanned<'a, &'a str>),
    File(Spanned<'a, &'a str>),
    Unit(Spanned<'a, &'a str>),
}

#[derive(Debug, Clone)]
pub struct FuncArg<'a> {
    pub name: SpannedStr<'a>,
    pub ty: PType<'a>,
}
