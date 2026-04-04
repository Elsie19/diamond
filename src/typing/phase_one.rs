//! # Phase One
//!
//! ### Notes
//! 1. All function definitions are globally scoped.

use crate::{parse::types::SpannedPVal, typing::types::Type};

pub struct Typing<'a> {
    program: &'a [SpannedPVal<'a>],
}
