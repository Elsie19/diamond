use typed_builder::TypedBuilder;

use crate::parse::types::{BPVal, Spanned, SpannedStr};

#[derive(Debug, Clone, TypedBuilder)]
pub struct For<'a> {
    loop_: Spanned<'a, PForInner<'a>>,
    body: BPVal<'a>,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct PForInner<'a> {
    pub bind: SpannedStr<'a>,
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
        &self.bind.node
    }
}
