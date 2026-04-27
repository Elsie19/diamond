use crate::interpreter::{engine::Engine, types::ILitType};

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
pub fn trim(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::String(str)] = args else {
        unreachable!("type checked");
    };

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
pub fn trim_left(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::String(str)] = args else {
        unreachable!("type checked");
    };

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
pub fn trim_right(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::String(str)] = args else {
        unreachable!("type checked");
    };

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
pub fn upper(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::String(str)] = args else {
        unreachable!("type checked");
    };

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
pub fn lower(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::String(str)] = args else {
        unreachable!("type checked");
    };

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
pub fn replace(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 3);

    let [
        ILitType::String(str),
        ILitType::String(from),
        ILitType::String(to),
    ] = args
    else {
        unreachable!("type checked");
    };

    Some(ILitType::String(str.replace(&**from, to).into()))
}
