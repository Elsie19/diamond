use std::ops::Deref;

pub type BPVal<'a> = Spanned<'a, Box<PVal<'a>>>;
pub type BPArr<'a> = Box<[Spanned<'a, PVal<'a>>]>;
pub type SpannedStr<'a> = Spanned<'a, &'a str>;

pub type SpannedPVal<'a> = Spanned<'a, PVal<'a>>;

#[derive(Debug, Clone)]
pub struct Spanned<'a, T> {
    pub node: T,
    pub span: pest::Span<'a>,
}

impl<'a, T> Deref for Spanned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, T> Spanned<'a, T> {
    pub fn new(node: T, span: pest::Span<'a>) -> Self {
        Self { node, span }
    }

    pub fn into_boxed(self) -> Spanned<'a, Box<T>> {
        Spanned {
            node: Box::new(self.node),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PVal<'a> {
    Atomic(Spanned<'a, PAtomic<'a>>),
    FuncCall {
        name: SpannedStr<'a>,
        args: BPArr<'a>,
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
        var: SpannedStr<'a>,
        iter: BPVal<'a>,
        body: BPArr<'a>,
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

impl<'a> PVal<'a> {
    pub unsafe fn atomic_unchecked(self) -> Spanned<'a, PAtomic<'a>> {
        match self {
            Self::Atomic(atomic) => atomic,
            _ => unreachable!("not `PVal::Atomic`"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PAtomic<'a> {
    Integer(Spanned<'a, usize>),
    String(Spanned<'a, &'a str>),
    Array(Spanned<'a, BPArr<'a>>),
    Ident(Spanned<'a, &'a str>),
}

impl<'a> PAtomic<'a> {
    pub fn span(&self) -> pest::Span<'a> {
        match self {
            Self::Array(a) => a.span,
            Self::Ident(i) => i.span,
            Self::String(s) => s.span,
            Self::Integer(i) => i.span,
        }
    }

    pub unsafe fn ident_unchecked(self) -> Spanned<'a, &'a str> {
        match self {
            Self::Ident(ident) => ident,
            _ => unreachable!("not `PAtomic::Ident`"),
        }
    }
}
