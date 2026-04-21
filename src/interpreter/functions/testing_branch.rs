use crate::interpreter::{
    engine::Engine,
    types::{ILitType, IResultBranch},
};

pub fn testing_branch(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let ILitType::Integer(toggle) = &args[0] else {
        unreachable!("type checked");
    };

    if *toggle == 0 {
        Some(ILitType::Result(IResultBranch::Ok(Box::new(
            ILitType::Integer(1),
        ))))
    } else {
        Some(ILitType::Result(IResultBranch::Err(Box::new(
            ILitType::Integer(1),
        ))))
    }
}
