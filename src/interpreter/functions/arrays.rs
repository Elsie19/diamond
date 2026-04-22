use crate::interpreter::{
    engine::Engine,
    types::{ILitType, IResultBranch, IStreamHandle},
};

/// Get value from array based on index.
///
/// # Signature
/// ```
/// let ~internal nth(arr: [any], nth: integer): result(any, string);
/// ```
///
/// # Details
/// Will return an `err` if out of bounds.
///
/// # Example
/// ```
/// let my_arr = ["hello", "world", "!"];
/// printf("This is my %s\n", [nth(my_arr, 1)!]);
/// ```
///
/// ```text
/// This is my world
/// ```
pub fn nth(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Array(arr), ILitType::Integer(nth)] = args else {
        unreachable!("type checked");
    };

    let elem = arr.get(*nth);

    match elem {
        Some(found) => Some(ILitType::Result(IResultBranch::Ok(Box::new(found.clone())))),
        None => Some(ILitType::Result(IResultBranch::Err(Box::new(
            ILitType::String(format!("invalid index `{}`", nth)),
        )))),
    }
}

/// Split an array by a pattern.
///
/// # Signature
/// ```
/// let ~internal split(string: string, char: string): [string];
/// ```
///
/// # Details
///
/// `split` is guaranteed to have at least one element, that being `string` if `char` could not
/// split by a pattern.
///
/// # Example
/// ```
/// let my_string = "Mary had a little lamb";
/// let split = split(my_string, " ");
/// for (part in split) {
///     printf("%s\n", [part]);
/// }
/// ```
///
/// ```text
/// Mary
/// had
/// a
/// little
/// lamb
/// ```
pub fn split(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::String(string), ILitType::String(char)] = args else {
        unreachable!("type checked");
    };

    let split = string
        .split(char)
        .map(|s| ILitType::String(s.to_string()))
        .collect::<Vec<_>>();

    Some(ILitType::Array(split.into_boxed_slice()))
}

/// Get length of parameter.
///
/// # Signature
/// ```
/// let ~internal len(probs_arr: any): integer;
/// ```
///
/// # Details
/// Because `len` takes `any`, it has to juggle every type:
///
/// |    Type |       Return Value      |        Details        |
/// |:-------:|:-----------------------:|:---------------------:|
/// | Array   | The length of the array |                     - |
/// | Integer |    The integer itself   |                     - |
/// | String  |      String length      |                     - |
/// | Unit    | `0`                     |        Units are null |
/// | Result  | `1`                     | Based on branch count |
/// | Stream  | Length of file in bytes |                     - |
/// | File    | Length of file path     |                     - |
///
/// # Example
/// ```
/// let my_string = "Mary had a little lamb";
/// let split = split(my_string, " ");
/// printf("%d\n", [len(split)]);
/// ```
///
/// ```text
/// 4
/// ```
pub fn len(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    Some(ILitType::Integer(match &args[0] {
        ILitType::Array(arr) => arr.len(),
        ILitType::Integer(int) => *int,
        ILitType::String(str) => str.len(),
        ILitType::Unit => 0,
        ILitType::Result(_) => 1,
        ILitType::Stream(f) => match f {
            IStreamHandle::File(handle) => handle.borrow().metadata().unwrap().len() as usize,
            _ => todo!("not yet kitten whiskers"),
        },
        ILitType::File(f) => f.as_os_str().len(),
    }))
}

/// Enumerate through values, giving an index with the value.
///
/// # Signature
/// ```
/// let ~internal enumerate(arr: [any]): [[any]];
/// ```
///
/// # Details
/// `enumerate` takes in an array of anything, and will return a arrays within a single outer
/// array. The inner array has only two elements, the index, and the value. For instance, given
/// the array `["hello", "world", "!"]`, `enumerate` will produce:
///
/// ```
/// [
///     [0, "hello"],
///     [1, "world"],
///     [2, "!"],
/// ]
/// ```
///
/// # Example
/// ```
/// let my_string = "Mary had a little lamb";
/// let split = split(my_string, " ");
/// for (idx_elem in enumerate(split)) {
///    let idx = nth(idx_elem, 0)!;
///    let elem = nth(idx_elem, 1)!;
///
///    printf("%d: %s\n", [idx, elem]);
/// };
/// ```
pub fn enumerate(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::Array(iter)] = args else {
        unreachable!("type checked");
    };

    let mut vec = Vec::with_capacity(iter.len());

    for (idx, elem) in iter.iter().enumerate() {
        vec.push(ILitType::Array(Box::new([
            ILitType::Integer(idx),
            elem.clone(),
        ])));
    }

    Some(ILitType::Array(vec.into_boxed_slice()))
}

/// Get last element of array.
///
/// # Signature
/// ```
/// let last(lst: [any]): any;
/// ```
///
/// # Example
/// ```
/// let lst = [1, 2, 3, 4];
/// printf("%d\n", [last(lst)]);
/// ```
///
/// ```
/// 4
/// ```
pub fn last(_lst: &[ILitType]) -> ILitType {
    ILitType::Unit
}
