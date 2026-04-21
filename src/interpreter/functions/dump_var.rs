use crate::interpreter::{engine::Engine, types::ILitType};

pub fn dump_var<'a>(_engine: &mut Engine<'a>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    println!("{arg:?}");

    Some(ILitType::Unit)
}
