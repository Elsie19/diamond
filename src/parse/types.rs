pub mod for_;
pub mod funccall;
pub mod funclet;
pub mod grouping;
pub mod let_;
pub mod match_;

use std::ops::Deref;

use enum_as_inner::EnumAsInner;

use crate::parse::grammar::MietteSpan;

/// A [`PVal`] with a span.
pub type BPVal<'a> = Spanned<'a, Box<PVal<'a>>>;
/// A [`Spanned`] array of [`PVal`]s.
pub type BPArr<'a> = Spanned<'a, Box<[Spanned<'a, PVal<'a>>]>>;

/// A [`Spanned<Pval>`].
pub type SpannedPVal<'a> = Spanned<'a, PVal<'a>>;

/// A [`Spanned`] string.
pub type SpannedStr<'a> = Spanned<'a, &'a str>;

/// A wrapped around a `T` with a span for error messages and diagnostics.
///
/// ```text
/// "foobar"
/// ^~~~~~~^ = Spanned::new("foobar", Span::new(r#""foobar""#, 1, 9).unwrap());
/// 12345678
/// ```
#[derive(Debug, Clone)]
pub struct Spanned<'a, T> {
    /// The `T` value.
    pub node: T,
    /// A span.
    pub span: pest::Span<'a>,
}

impl<T> Deref for Spanned<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, T> MietteSpan for Spanned<'a, T> {
    fn as_miette_span(&self) -> miette::SourceSpan {
        miette::SourceSpan::from(self.span().start()..self.span().end())
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

impl<T> IntoIterator for Spanned<'_, T>
where
    T: IntoIterator,
{
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.node.into_iter()
    }
}

impl<'a, T> Spanned<'a, T> {
    /// Create a new spanned object.
    pub fn new(node: T, span: pest::Span<'a>) -> Self {
        Self { node, span }
    }

    /// Consume the span and get the inner `T`.
    pub fn into_inner(self) -> T {
        self.node
    }

    pub fn as_inner(&self) -> &T {
        &self.node
    }

    /// [`Box`] the inner `T`.
    pub fn into_boxed(self) -> Spanned<'a, Box<T>> {
        Spanned {
            node: Box::new(self.node),
            span: self.span,
        }
    }

    /// Get the span of the object.
    #[must_use = "use the span bruh"]
    pub const fn span(&self) -> pest::Span<'a> {
        self.span
    }

    pub fn pretty_span(&self) -> String {
        format!("[{}..{}]", self.span().start(), self.span().end())
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum PVal<'a> {
    Atomic(Spanned<'a, PAtomic<'a>>),
    FuncCall(funccall::FuncCall<'a>),
    FuncLet(funclet::FuncLet<'a>),
    Grouping(grouping::Grouping<'a>),
    Match(match_::Match<'a>),
    For(for_::For<'a>),
    Let(let_::Let<'a>),
    Expr(BPVal<'a>),
    Stmt(BPVal<'a>),
}

impl PVal<'_> {
    pub fn into_name(self) -> &'static str {
        match self {
            Self::For { .. } => "For",
            Self::Let { .. } => "Let",
            Self::Expr(_) => "Expr",
            Self::Stmt(_) => "Stmt",
            Self::Atomic(_) => "Atomic",
            Self::Match { .. } => "Match",
            Self::FuncLet { .. } => "FuncLet",
            Self::FuncCall { .. } => "FuncCall",
            Self::Grouping { .. } => "Grouping",
        }
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum PAtomic<'a> {
    Integer(Spanned<'a, usize>),
    String(Spanned<'a, &'a str>),
    Array(Spanned<'a, BPArr<'a>>),
    Ident(Spanned<'a, &'a str>),
    Unit(Spanned<'a, &'a str>),
    Result(Spanned<'a, (Box<Self>, Box<Self>)>),
}

impl<'a> PAtomic<'a> {
    pub const fn span(&self) -> pest::Span<'a> {
        match self {
            Self::Array(a) => a.span,
            Self::Ident(i) => i.span,
            Self::String(s) => s.span,
            Self::Integer(i) => i.span,
            Self::Unit(u) => u.span,
            Self::Result(r) => r.span,
        }
    }

    pub const fn kind(&self) -> &'static str {
        match self {
            PAtomic::Integer(_) => "Integer",
            PAtomic::String(_) => "String",
            PAtomic::Array(_) => "Array",
            PAtomic::Ident(_) => "Ident",
            PAtomic::Unit(_) => "Unit",
            PAtomic::Result(_) => "Result",
        }
    }

    pub fn pretty_span(&self) -> String {
        match self {
            PAtomic::Integer(spanned) => spanned.pretty_span(),
            PAtomic::String(spanned) => spanned.pretty_span(),
            PAtomic::Array(spanned) => spanned.pretty_span(),
            PAtomic::Ident(spanned) => spanned.pretty_span(),
            PAtomic::Unit(spanned) => spanned.pretty_span(),
            PAtomic::Result(spanned) => spanned.pretty_span(),
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
    Unret(Spanned<'a, &'a str>),
    Integer(Spanned<'a, &'a str>),
    Result(Spanned<'a, (Box<Self>, Box<Self>)>),
}

impl<'a> PType<'a> {
    pub const fn span(&self) -> pest::Span<'a> {
        match self {
            Self::Array(a) => a.span,
            Self::String(s) => s.span,
            Self::Integer(i) => i.span,
            Self::Unit(u) => u.span,
            Self::Result(r) => r.span,
            Self::Stream(s) => s.span,
            Self::Unret(u) => u.span,
            Self::File(f) => f.span,
        }
    }
}
