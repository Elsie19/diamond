use typed_builder::TypedBuilder;

use crate::parse::types::{BPVal, PVal, Spanned};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Grouping<'a> {
    stmts: Box<[Spanned<'a, PVal<'a>>]>,
    #[builder(default=None, setter(strip_option))]
    redirect: Option<BPVal<'a>>,
}

impl<'a> Grouping<'a> {
    pub fn stmts_raw(&self) -> &Box<[Spanned<'a, PVal<'a>>]> {
        &self.stmts
    }

    pub fn redirect_raw(&self) -> &Option<BPVal<'a>> {
        &self.redirect
    }
}
