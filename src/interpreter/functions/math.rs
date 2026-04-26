use crate::interpreter::{engine::Engine, types::ILitType};

/// Get max of two numbers.
///
/// # Signature
/// ```
/// let ~internal max(fst: integer, snd: integer): integer;
/// ```
///
/// # Example
/// ```
/// let max = max(50, 100);
/// printf("max is %d\n", [max]);
/// ```
///
/// ```text
/// max is 100
/// ```
pub fn max(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Integer(fst), ILitType::Integer(snd)] = args else {
        unreachable!("type checked");
    };

    Some(ILitType::Integer(*fst.max(snd)))
}

/// Get min of two numbers.
///
/// # Signature
/// ```
/// let ~internal min(fst: integer, snd: integer): integer;
/// ```
///
/// # Example
/// ```
/// let min = min(50, 100);
/// printf("min is %d\n", [max]);
/// ```
///
/// ```text
/// min is 50
/// ```
pub fn min(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Integer(fst), ILitType::Integer(snd)] = args else {
        unreachable!("type checked");
    };

    Some(ILitType::Integer(*fst.min(snd)))
}

/// Add two numbers.
///
/// # Signature
/// ```
/// let ~internal add(fst: integer, snd: integer): integer;
/// ```
///
/// # Details
/// Will wrap around if sum is past [`usize::MAX`].
///
/// # Example
/// ```
/// let sum = add(50, 100);
/// printf("50 + 100 = %d\n", [sum]);
/// ```
///
/// ```text
/// 50 + 100 = 150
/// ```
pub fn add(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Integer(fst), ILitType::Integer(snd)] = args else {
        unreachable!("type checked");
    };

    Some(ILitType::Integer(fst.wrapping_add(*snd)))
}

/// Subtract two numbers.
///
/// # Signature
/// ```
/// let ~internal sub(fst: integer, snd: integer): integer;
/// ```
///
/// # Details
/// Will wrap around if result is under [`usize::MIN`].
///
/// # Example
/// ```
/// let sub = sub(100, 50);
/// printf("100 - 50 = %d\n", [sub]);
/// ```
///
/// ```text
/// 100 - 50 = 50
/// ```
pub fn sub(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Integer(fst), ILitType::Integer(snd)] = args else {
        unreachable!("type checked");
    };

    Some(ILitType::Integer(fst.wrapping_sub(*snd)))
}
