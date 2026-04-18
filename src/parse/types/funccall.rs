use typed_builder::TypedBuilder;

use crate::parse::types::{BPArr, BPVal, Spanned};

#[derive(Debug, Clone, TypedBuilder)]
pub struct FuncCall<'a> {
    name: BPVal<'a>,
    #[builder(default=None, setter(strip_option))]
    args: Option<BPArr<'a>>,
    #[builder(default=None, setter(strip_option))]
    unwrap: Option<Spanned<'a, bool>>,
}

impl<'a> FuncCall<'a> {
    pub fn has_unwrap(&self) -> bool {
        match &self.unwrap {
            Some(span) => span.node,
            None => false,
        }
    }

    pub fn get_unwrap(&self) -> &Option<Spanned<'a, bool>> {
        &self.unwrap
    }

    pub fn args_raw(&self) -> &Option<BPArr<'a>> {
        &self.args
    }

    pub fn name_raw(&self) -> &BPVal<'a> {
        &self.name
    }

    pub fn name(&self) -> &str {
        unsafe {
            self.name
                .node
                .as_atomic_unchecked()
                .node
                .as_ident_unchecked()
                .node
        }
    }
}
