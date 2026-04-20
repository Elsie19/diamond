use typed_builder::TypedBuilder;

use crate::parse::types::{BPVal, PAtomic, Spanned, SpannedPVal, SpannedStr};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Match<'a> {
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    expr: BPVal<'a>,
    arms: Box<[Spanned<'a, PMatchArm<'a>>]>,
}

impl<'a> Match<'a> {
    pub fn expr_raw(&self) -> &BPVal<'a> {
        &self.expr
    }

    pub fn arms_raw(&self) -> &[Spanned<'a, PMatchArm<'a>>] {
        &self.arms
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct PMatchArm<'a> {
    /// The literal text `ok` or `err`.
    pub res: PMatchCase<'a>,
    /// The value associated with the branch.
    #[builder(setter(transform = |x: PAtomic<'a>| unsafe { x.into_ident_unchecked() }))]
    pub inner: SpannedStr<'a>,
    /// The code that executes if matched.
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    pub expr: BPVal<'a>,
}

impl PMatchArm<'_> {
    pub const fn ok(&self) -> bool {
        matches!(self.res, PMatchCase::Ok(_))
    }

    pub const fn err(&self) -> bool {
        matches!(self.res, PMatchCase::Err(_))
    }
}

#[derive(Debug, Clone)]
pub enum PMatchCase<'a> {
    Ok(SpannedStr<'a>),
    Err(SpannedStr<'a>),
}

impl<'a> PMatchCase<'a> {
    pub fn name(&self) -> &str {
        match self {
            PMatchCase::Ok(spanned) | PMatchCase::Err(spanned) => &*spanned
        }
    }
}
