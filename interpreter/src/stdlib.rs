use std::collections::HashMap;

use crate::functions;

use type_checker::{strata::IR, types::Type};

use crate::{engine::Engine, types::ILitType};

macro_rules! stdlib {
    (
        $(
            $name:ident => $path:path
        ),* $(,)?
    ) => {{
        ::std::collections::HashMap::<
            &str,
            $crate::stdlib::RuntimeFunc
        >::from([
            $(
                (
                    stringify!($name),
                    $crate::stdlib::RuntimeFunc::Internal($path),
                ),
            )*
        ])
    }};
}

#[derive(Debug, Clone)]
pub enum RuntimeFunc<'a> {
    User(UserFunc<'a>),
    Internal(fn(&mut Engine<'a>, &[ILitType]) -> ILitType),
}

#[derive(Debug, Clone)]
pub struct UserFunc<'a> {
    pub args: Box<[(usize, Type)]>,
    pub body: &'a IR,
    pub ret: Type,
}

#[derive(Debug)]
pub struct Functions<'a> {
    funcs: HashMap<&'a str, RuntimeFunc<'a>>,
}

impl<'a> Functions<'a> {
    pub fn stdlib() -> Self {
        Self {
            funcs: stdlib! {
                itoa => functions::itoa::itoa,
                atoi => functions::itoa::atoi,

                max => functions::math::max,
                min => functions::math::min,
                add => functions::math::add,
                sub => functions::math::sub,

                dump_var => functions::dump_var::dump_var,

                sprintf => functions::printf::sprintf,
                printf =>  functions::printf::printf,
                puts =>    functions::printf::puts,

                nth =>       functions::arrays::nth,
                split =>     functions::arrays::split,
                len =>       functions::arrays::len,
                enumerate => functions::arrays::enumerate,
                chars =>     functions::arrays::chars,
                only =>      functions::arrays::only,
                rev =>       functions::arrays::rev,

                file =>   functions::file::file,
                create => functions::file::create,
                open =>   functions::file::open,
                dump =>   functions::file::dump,
                lines =>  functions::file::lines,
                skip =>   functions::file::skip,
                fpop =>   functions::file::fpop,

                panic => functions::system::panic,
                exit =>  functions::system::exit,
                args =>  functions::system::args,

                ok =>  functions::result::ok,
                err => functions::result::err,

                trim =>       functions::strings::trim,
                trim_left =>  functions::strings::trim_left,
                trim_right => functions::strings::trim_right,
                upper =>      functions::strings::upper,
                lower =>      functions::strings::lower,
                replace =>    functions::strings::replace,
                split_at =>   functions::strings::split_at,
            },
        }
    }

    pub fn insert(&mut self, name: &'a str, func: RuntimeFunc<'a>) {
        self.funcs.insert(name, func);
    }

    pub fn resolve<S: AsRef<str>>(&self, name: S) -> Option<&RuntimeFunc<'a>> {
        self.funcs.get(name.as_ref())
    }
}
