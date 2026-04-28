use typed_builder::TypedBuilder;

use crate::types::{BPVal, PAtomic, SpannedPVal, SpannedStr};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Let<'a> {
    #[builder(setter(transform = |x: PAtomic<'a>| unsafe { x.into_ident_unchecked() }))]
    name: SpannedStr<'a>,
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    expr: BPVal<'a>,
}

impl<'a> Let<'a> {
    pub fn name_raw(&self) -> &SpannedStr<'a> {
        &self.name
    }

    pub fn name(&self) -> &str {
        self.name.node
    }

    pub fn expr_raw(&self) -> &BPVal<'a> {
        &self.expr
    }
}
