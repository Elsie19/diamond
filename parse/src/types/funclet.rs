use typed_builder::TypedBuilder;

use crate::types::{BPVal, PAtomic, PType, Spanned, SpannedPVal, SpannedStr};

#[derive(Debug, Clone, TypedBuilder)]
pub struct FuncLet<'a> {
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    name: BPVal<'a>,
    #[builder(default=None, setter(strip_option))]
    args: Option<Spanned<'a, Box<[FuncArg<'a>]>>>,
    #[builder(default=None, setter(strip_option))]
    ret: Option<PType<'a>>,
    #[builder(setter(transform = |x: SpannedPVal<'a>| x.into_boxed()))]
    body: BPVal<'a>,
    #[builder(default = false)]
    internal: bool,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct FuncArg<'a> {
    #[builder(setter(transform = |x: PAtomic<'a>| unsafe { x.into_ident_unchecked() }))]
    pub name: SpannedStr<'a>,
    pub ty: PType<'a>,
}

impl<'a> FuncLet<'a> {
    pub fn is_internal(&self) -> bool {
        self.internal
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

    pub fn args_raw(&self) -> &Option<Spanned<'a, Box<[FuncArg<'a>]>>> {
        &self.args
    }

    pub fn args(&self) -> Option<&Spanned<'a, Box<[FuncArg<'a>]>>> {
        self.args.as_ref()
    }

    pub const fn args_len(&self) -> usize {
        match self.args {
            Some(ref args) => args.node.len(),
            None => 0,
        }
    }

    pub fn body_raw(&self) -> &BPVal<'a> {
        &self.body
    }

    pub fn ret_raw(&self) -> &Option<PType<'a>> {
        &self.ret
    }

    pub fn ret(&self) -> Option<&PType<'a>> {
        self.ret.as_ref()
    }

    pub fn into_args_ret(self) -> (Option<Spanned<'a, Box<[FuncArg<'a>]>>>, Option<PType<'a>>) {
        (self.args, self.ret)
    }
}
