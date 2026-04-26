use crate::{
    interpreter::{engine::Engine, types::ILitType},
    res,
};

/// Return an `ok` value.
///
/// # Signature
/// ```
/// let ~internal ok(val: any): result(any, any);
/// ```
///
/// # Example
/// ```
/// let ok_val = ok([1, 2, 3]);
/// ```
pub fn ok(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    Some(ILitType::Result(res!(Ok, any => arg.clone())))
}

/// Return an `err` value.
///
/// # Signature
/// ```
/// let ~internal err(val: any): result(any, any);
/// ```
///
/// # Example
/// ```
/// let oops = err("failed to do something");
/// ```
pub fn err(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    Some(ILitType::Result(res!(Err, any => arg.clone())))
}
