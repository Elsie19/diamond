use sig_macro::signature;

use crate::{engine::Engine, res, types::ILitType};

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

/// Checks for equality.
///
/// # Signature
/// ```
/// let ~internal eq(lhs: any, rhs: any): result(unit, unit);
/// ```
///
/// # Example
/// ```
/// let a = "a";
/// let b = "b";
/// match (eq(a, b)) {
///   ok o => printf("Equal!\n", []),
///   err e => printf("Not equal!\n", []),
/// }
/// ```
#[signature(args => lhs: any, rhs: any)]
pub fn eq(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(if *lhs == *rhs {
        res!(Ok, unit)
    } else {
        res!(Err, unit)
    })
}
