use sig_macro::signature;

use crate::{engine::Engine, res, types::ILitType};

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
#[signature(args => num: integer)]
pub fn itoa(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(num.to_string().into())
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
#[signature(args => num: string)]
pub fn atoi(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(match num.parse::<usize>() {
        Ok(num) => res!(Ok, int_dy => num),
        Err(e) => res!(Err, str_dy => e.to_string()),
    })
}
