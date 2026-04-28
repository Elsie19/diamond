use sig_macro::signature;

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
#[signature(args => val: any)]
pub fn ok(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(res!(Ok, any => val.clone()))
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
#[signature(args => val: any)]
pub fn err(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(res!(Err, any => val.clone()))
}
