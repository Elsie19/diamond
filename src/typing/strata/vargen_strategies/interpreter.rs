use std::collections::HashSet;

use crate::typing::strata::VarGenerator;

/// Simple variable generator strategy.
///
/// It will normalize variables by replacing `-` with `_` and will simply append `_{number}` to the
/// end.
#[derive(Debug, Default)]
pub struct VarGenInterpreter {
    store: HashSet<String>,
}

impl VarGenerator for VarGenInterpreter {
    fn init() -> Self {
        Self::default()
    }

    fn fresh<S>(&mut self, str: S) -> String
    where
        S: AsRef<str>,
    {
        let mut num = 0;
        loop {
            num += 1;
            let id = format!("{}_{}", Self::normalize(str.as_ref()), num);

            if self.store.insert(id.clone()) {
                return id;
            }
        }
    }
}

impl VarGenInterpreter {
    fn normalize(str: &str) -> String {
        str.replace('-', "_")
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect()
    }
}
