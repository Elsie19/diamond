use std::rc::Rc;

use shared::unreachable_unchecked;
use sig_macro::signature;

use crate::{engine::Engine, functions::printf::sprintf, types::ILitType};

/// Exit with code.
///
/// # Signature
/// ```
/// let ~internal exit(code: integer): unret;
/// ```
#[signature(args => code: integer)]
pub fn exit(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    std::process::exit(*code as i32)
}

/// Panic with message.
///
/// # Signature
/// ```
/// let ~internal panic(msg: string): unret;
/// ```
pub fn panic(engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    let ILitType::String(s) = sprintf(engine, args) else {
        unreachable_unchecked!()
    };

    eprintln!("thread 'main' ({}) panicked: {}", std::process::id(), s);

    std::process::exit(1)
}

/// Get arguments from command line.
///
/// # Signature
/// ```
/// let ~internal args(): [string];
/// ```
pub fn args(engine: &mut Engine<'_>, _args: &[ILitType]) -> ILitType {
    let args = engine.args();

    ILitType::Array(Rc::clone(args))
}
