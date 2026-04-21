use crate::interpreter::{engine::Engine, types::ILitType};

pub fn itoa(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    if let ILitType::Integer(int) = arg {
        Some(ILitType::String(int.to_string()))
    } else {
        None
    }
}
