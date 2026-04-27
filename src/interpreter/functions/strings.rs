use sig_macro::signature;

use crate::{
    interpreter::{engine::Engine, types::ILitType},
    res,
};

/// Trim string.
///
/// # Signature
/// ```
/// let ~internal trim(str: string): string;
/// ```
///
/// # Example
/// ```
/// let string = "   hello   ";
/// let trimmed = trim(string);
/// printf("%d\n", [len(trimmed)]);
/// ```
///
/// ```
/// 4
/// ```
#[signature(args => str: string)]
pub fn trim(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    Some(ILitType::String(str.trim().into()))
}

/// Trim left side of string.
///
/// # Signature
/// ```
/// let ~internal trim_left(str: string): string;
/// ```
///
/// # Example
/// ```
/// let string = "   hello";
/// let trimmed = trim_left(string);
/// printf("%d\n", [len(trimmed)]);
/// ```
///
/// ```
/// 4
/// ```
#[signature(args => str: string)]
pub fn trim_left(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    Some(ILitType::String(str.trim_start().into()))
}

/// Trim right side of string.
///
/// # Signature
/// ```
/// let ~internal trim_right(str: string): string;
/// ```
///
/// # Example
/// ```
/// let string = "hello   ";
/// let trimmed = trim_right(string);
/// printf("%d\n", [len(trimmed)]);
/// ```
///
/// ```
/// 4
/// ```
#[signature(args => str: string)]
pub fn trim_right(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    Some(ILitType::String(str.trim_end().into()))
}

/// Uppercase a string.
///
/// # Signature
/// ```
/// let ~internal upper(str: string): string;
/// ```
///
/// # Example
/// ```
/// let string = "hello";
/// let upper = upper(string);
/// printf("%s\n", [upper]);
/// ```
///
/// ```text
/// HELLO
/// ```
#[signature(args => str: string)]
pub fn upper(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    Some(ILitType::String(str.to_uppercase().into()))
}

/// Lowercase a string.
///
/// # Signature
/// ```
/// let ~internal lower(str: string): string;
/// ```
///
/// # Example
/// ```
/// let string = "HELLO";
/// let lower = lower(string);
/// printf("%s\n", [lower]);
/// ```
///
/// ```text
/// hello
/// ```
#[signature(args => str: string)]
pub fn lower(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    Some(ILitType::String(str.to_lowercase().into()))
}

/// Replace a pattern with text.
///
/// # Signature
/// ```
/// let ~internal replace(str: string, from: string, to: string): string;
/// ```
///
/// # Example
/// ```
/// let string = "Hey Bob!";
/// let replaced = replace(string, "Bob", "Joe");
/// printf("%s\n", [replaced]);
/// ```
///
/// ```text
/// Hey Joe!
/// ```
#[signature(args => str: string, from: string, to: string)]
pub fn replace(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    Some(ILitType::String(str.replace(&**from, to).into()))
}

/// Split at point in string.
///
/// # Signature
/// ```
/// let ~internal split_at(str: string, mid: integer): result([string], string);
/// ```
///
/// # Example
/// ```
/// let string = "Hey Bob!";
/// let replaced = split_at(string, 3)!;
/// let fst = nth(replaced, 0)!;
/// let snd = nth(replaced, 1)!;
/// printf("%s%s\n", [fst, snd]);
/// ```
///
/// ```text
/// Hey Bob!
/// ```
#[signature(args => str: string, mid: integer)]
pub fn split_at(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let split_at = str.split_at_checked(*mid);

    Some(ILitType::Result(match split_at {
        Some((a, b)) => res!(Ok, arr => [ILitType::String(a.into()), ILitType::String(b.into())]),
        None => res!(Err, str => "invalid byte offset"),
    }))
}
