use std::collections::HashMap;

use crate::{
    interpreter::{engine::Engine, functions, types::ILitType},
    typing::strata::IR,
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
pub(crate) enum RuntimeFunc<'a> {
    User(&'a IR),
    Internal(fn(&mut Engine<'a>, &[ILitType]) -> Option<ILitType>),
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
            },
        }
    }

    pub fn resolve<S: AsRef<str>>(&self, name: S) -> Option<&RuntimeFunc<'a>> {
        self.funcs.get(name.as_ref())
    }
}
