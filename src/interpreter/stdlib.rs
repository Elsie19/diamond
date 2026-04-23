use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{engine::Engine, functions, types::ILitType},
    typing::{strata::IR, types::Type},
};

macro_rules! stdlib {
    (
        $(
            $name:ident => $path:path
        ),* $(,)?
    ) => {{
        let mut map: std::collections::HashMap<String, RuntimeFunc<'a>> =
            std::collections::HashMap::new();

        $(
            map.insert(
                stringify!($name).to_string(),
                RuntimeFunc::Internal($path),
            );
        )*

        map
    }};
}

#[derive(Debug, Clone)]
pub enum RuntimeFunc<'a> {
    User(UserFunc<'a>),
    Internal(fn(&mut Engine<'a>, &[ILitType]) -> Option<ILitType>),
}

#[derive(Debug, Clone)]
pub struct UserFunc<'a> {
    pub args: Box<[(Rc<str>, Type)]>,
    pub body: &'a [IR],
    pub ret: Type,
}

#[derive(Debug)]
pub struct Functions<'a> {
    funcs: HashMap<String, RuntimeFunc<'a>>,
}

impl<'a> Functions<'a> {
    pub fn stdlib() -> Self {
        Self {
            funcs: stdlib! {
                itoa => functions::itoa::itoa,

                dump_var => functions::dump_var::dump_var,

                sprintf => functions::printf::sprintf,
                printf =>  functions::printf::printf,
                puts =>    functions::printf::puts,

                panic => functions::panic::panic,

                testing_branch => functions::testing_branch::testing_branch,

                nth =>         functions::arrays::nth,
                split =>       functions::arrays::split,
                len =>         functions::arrays::len,
                enumerate =>   functions::arrays::enumerate,

                file =>   functions::file::file,
                create => functions::file::create,
                open =>   functions::file::open,
                dump =>   functions::file::dump,
                lines =>  functions::file::lines,
            },
        }
    }

    pub fn insert<T: ToString>(&mut self, name: &T, func: RuntimeFunc<'a>) {
        self.funcs.insert(name.to_string(), func);
    }

    pub fn resolve<S: AsRef<str>>(&self, name: S) -> Option<&RuntimeFunc<'a>> {
        self.funcs.get(name.as_ref())
    }
}
