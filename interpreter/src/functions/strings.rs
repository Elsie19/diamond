use collect_into_rc_slice::CollectIntoRcSlice;
use sig_macro::signature;

use crate::{engine::Engine, res, types::ILitType};

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
pub fn trim(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(str.trim().into())
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
pub fn trim_left(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(str.trim_start().into())
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
pub fn trim_right(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(str.trim_end().into())
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
pub fn upper(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(str.to_uppercase().into())
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
pub fn lower(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(str.to_lowercase().into())
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
pub fn replace(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(str.replace(&**from, to).into())
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
pub fn split_at(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    let split_at = str.split_at_checked(*mid);

    ILitType::Result(match split_at {
        Some((a, b)) => res!(Ok, arr => [ILitType::String(a.into()), ILitType::String(b.into())]),
        None => res!(Err, str => "invalid byte offset"),
    })
}

/// Join a string array with a character.
///
/// # Signature
/// ```
/// let ~internal join(arr: [string], join: string): string;
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
#[signature(args => arr: [string], join: string)]
pub fn join(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::String(
        arr.iter()
            .map(|arg| unsafe { arg.clone().into_string_unchecked() })
            .collect_into_rc_slice()
            .join(join)
            .into(),
    )
}

/// Check if, in a string, at position `at`, any `patterns` exist.
///
/// # Signature
/// ```
/// let ~internal pattern_pos(str: string, at: integer, patterns: [string]): result(integer,
/// integer);
/// ```
///
/// # Example
/// ```
/// let string = "Hey Jill!";
/// let check = pattern_pos(string, 0, ["Hey"]);
/// match (check) {
///   ok o = puts("string starts with 'Hey'"),
///   err e = puts("string does not start with 'Hey'"),
/// }
/// ```
///
/// ```text
/// string starts with 'Hey'
/// ```
#[signature(args => str: string, at: integer, patterns: [string])]
pub fn pattern_pos(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    // SAFETY: We already checked with [`signature`] that `patterns` is all strings underneath.
    let mut patterns = patterns.iter().map(|s| unsafe { s.as_string_unchecked() });

    ILitType::Result(
        if str
            .get(*at..)
            .is_some_and(|rest| patterns.any(|pat| rest.starts_with(&**pat)))
        {
            res!(Ok, int_dy => *at)
        } else {
            res!(Err, int_dy => *at)
        },
    )
}
