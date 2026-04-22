use crate::interpreter::{
    engine::Engine,
    types::{ILitType, IResultBranch},
};

// let ~internal nth(arr: [any], nth: integer): result(any, string);
pub fn nth(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Array(arr), ILitType::Integer(nth)] = args else {
        unreachable!("type checked");
    };

    let elem = arr.get(*nth);

    match elem {
        Some(found) => Some(ILitType::Result(IResultBranch::Ok(Box::new(found.clone())))),
        None => Some(ILitType::Result(IResultBranch::Err(Box::new(
            ILitType::String(format!("invalid index `{}`", nth)),
        )))),
    }
}
