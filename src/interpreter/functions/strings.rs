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
