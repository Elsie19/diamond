use crate::interpreter::{engine::Engine, functions::printf::sprintf, types::ILitType};

/// Exit with code.
///
/// # Signature
/// ```
/// let ~internal exit(code: integer): unret;
/// ```
pub fn exit(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    if let ILitType::Integer(code) = arg {
        std::process::exit(*code as i32)
    } else {
        None
    }
}

/// Panic with message.
///
/// # Signature
/// ```
/// let ~internal panic(msg: string): unret;
/// ```
pub fn panic(engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let ret = sprintf(engine, args);

    let Some(ILitType::String(s)) = ret else {
        unreachable!("type checked");
    };

    eprint!("thread 'main' ({}) panicked: {}", std::process::id(), s);

    std::process::exit(1)
}
