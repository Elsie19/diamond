use std::{borrow::Cow, collections::HashSet, rc::Rc};

use crate::typing::strata::VarGenerator;

/// Simple variable generator strategy.
///
/// It will normalize variables by replacing `-` with `_` and will simply append `_{number}` to the
/// end.
#[derive(Debug, Default)]
pub struct VarGenInterpreter {
    store: HashSet<Rc<str>>,
}

impl VarGenerator for VarGenInterpreter {
    fn init() -> Self {
        Self::default()
    }

    fn fresh<S>(&mut self, str: S) -> Rc<str>
    where
        S: AsRef<str>,
    {
        let mut num = 0;
        loop {
            num += 1;
            let id = format!("{}_{}", Self::normalize(str.as_ref()), num);

            let id: Rc<str> = id.into();

            if self.store.insert(id.clone()) {
                return id;
            }
        }
    }
}

impl VarGenInterpreter {
    // I know that `format!` will erase the usefulness of Cow, but if I can reduce even one
    // allocation that'd be nice.
    fn normalize(str: &str) -> Cow<'_, str> {
        str.to_ascii_lowercase()
            .replace('-', "_")
            .chars()
            .filter(char::is_ascii_alphabetic)
            .collect()
    }
}
