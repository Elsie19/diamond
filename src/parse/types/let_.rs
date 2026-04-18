use typed_builder::TypedBuilder;

use crate::parse::types::{BPVal, SpannedStr};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Let<'a> {
    name: SpannedStr<'a>,
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
