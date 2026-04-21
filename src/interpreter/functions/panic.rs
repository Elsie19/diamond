use crate::interpreter::{engine::Engine, functions::printf::sprintf, types::ILitType};

pub fn panic<'a>(engine: &mut Engine<'a>, args: &[ILitType]) -> Option<ILitType> {
    let ret = sprintf(engine, args);

    let Some(ILitType::String(s)) = ret else {
        unreachable!("type checked");
    };

    eprint!("rt panic: {}", s);
    std::process::exit(1)
}
