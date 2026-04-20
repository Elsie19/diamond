use nanoid::nanoid;
use std::collections::HashSet;

use crate::typing::types::Type;

pub struct VarGen {
    store: HashSet<String>,
    valid_chars: [char; 36],
}

impl Default for VarGen {
    fn default() -> Self {
        Self {
            store: HashSet::new(),
            valid_chars: [
                '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
                'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
                'w', 'x', 'y', 'z',
            ],
        }
    }
}

impl VarGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn var(&mut self) -> &str {
        loop {
            let id = nanoid!(10, &self.valid_chars);

            if self.store.insert(id.clone()) {
                return self.store.get(&id).expect("checked above");
            }
        }
    }
}

#[derive(Debug)]
pub enum IR {
    FuncLet {
        name: String,
        args: Vec<(String, Type)>,
        internal: bool,
        ret: Type,
        body: Vec<Self>,
    },
    Grouping {
        inner: Vec<Self>,
        redirect: Option<Box<Self>>,
    }
}
