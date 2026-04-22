use std::path::PathBuf;

use crate::interpreter::{engine::Engine, types::ILitType};

pub fn file(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    if let ILitType::String(path) = arg {
        Some(ILitType::File(PathBuf::from(path)))
    } else {
        None
    }
}
