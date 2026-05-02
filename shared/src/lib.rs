pub mod bin_header;

use bincode::{Decode, Encode};
use type_checker::{pass_one::FuncTable, strata::IR};

#[derive(Decode, Encode)]
pub struct Bundle {
    pub ir: Vec<IR>,
    pub funcs: FuncTable,
}

#[macro_export]
macro_rules! unreachable_unchecked {
    () => {
        unsafe { ::std::hint::unreachable_unchecked() }
    };
}
