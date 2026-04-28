use typed_builder::TypedBuilder;

use crate::types::{BPVal, PAtomic, Spanned, SpannedPVal, SpannedStr};

#[derive(Debug, Clone, TypedBuilder)]
pub struct For<'a> {
    loop_: Spanned<'a, PForInner<'a>>,
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    body: BPVal<'a>,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct PForInner<'a> {
    #[builder(setter(transform = |x: PAtomic<'a>| unsafe { x.into_ident_unchecked() }))]
    pub bind: SpannedStr<'a>,
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    pub expr: BPVal<'a>,
}

impl<'a> For<'a> {
    pub fn loop_raw(&self) -> &Spanned<'a, PForInner<'a>> {
        &self.loop_
    }

    pub fn body_raw(&self) -> &BPVal<'a> {
        &self.body
    }
}

impl<'a> PForInner<'a> {
    pub fn bind_raw(&self) -> &SpannedStr<'a> {
        &self.bind
    }

    pub fn expr_raw(&self) -> &BPVal<'a> {
        &self.expr
    }

    pub fn bind(&self) -> &str {
        self.bind.node
    }
}
