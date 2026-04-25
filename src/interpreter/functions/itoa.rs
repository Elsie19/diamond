use crate::interpreter::{
    engine::Engine,
    types::{ILitType, IResultBranch},
};

/// Convert `integer` to `string`.
///
/// # Signature
/// ```
/// let ~internal itoa(num: integer): string;
/// ```
///
/// # Example
/// ```
/// let my_num = 99;
/// printf("My number as a string is: %s\n", [itoa(my_num)]);
/// ```
///
/// ```text
/// My number as a string is: 99
/// ```
pub fn itoa(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    if let ILitType::Integer(int) = arg {
        Some(ILitType::String(int.to_string().into()))
    } else {
        None
    }
}

/// Convert `string` to `integer`.
///
/// # Signature
/// ```
/// let ~internal atoi(num: string): result(integer, string);
/// ```
///
/// # Details
/// Will return an `err` if string cannot be parsed into a number.
///
/// # Example
/// ```
/// let my_num = "99";
/// let my_num_as_num = atoi(my_num)!;
/// printf("My number is: %d\n", [my_num_as_num]);
/// ```
///
/// ```text
/// My number is: 99
/// ```
pub fn atoi(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    let ILitType::String(num) = arg else {
        unreachable!("type checked");
    };

    Some(ILitType::Result(match num.parse::<usize>() {
        Ok(num) => IResultBranch::Ok(Box::new(ILitType::Integer(num))),
        Err(e) => IResultBranch::Err(Box::new(ILitType::String(e.to_string().into()))),
    }))
}
