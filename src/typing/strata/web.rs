use std::collections::HashMap;

use crate::typing::types::Type;

/// Variable mapper.
///
/// At this point in the process, there are no scopes, only unique IDs.
#[derive(Debug)]
pub struct Web {
    vars: VarGen,
    map: HashMap<usize, Type>,
}

impl Web {
    pub fn new() -> Self {
        Self {
            vars: VarGen::new(),
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, v: Type) {
        self.map.insert(self.vars.fresh(), v);
    }
}

#[derive(Debug)]
pub struct VarGen {
    ct: usize,
}

impl VarGen {
    pub fn new() -> Self {
        Self { ct: 0 }
    }

    pub fn fresh(&mut self) -> usize {
        let ret = self.ct;
        self.ct += 1;
        ret
    }
}
