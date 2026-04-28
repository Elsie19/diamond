use sig_macro::signature;

use crate::interpreter::{engine::Engine, types::ILitType};

#[signature(args => arg: any)]
pub fn dump_var(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    println!("{arg:?}");

    ILitType::Unit
}
