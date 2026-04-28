use std::rc::Rc;

use collect_into_rc_slice::CollectIntoRcSlice;
use sig_macro::signature;

use crate::{
    engine::Engine,
    res,
    types::{ILitType, IStreamHandle},
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
#[signature(args => arr: [any], nth: integer)]
pub fn nth(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    let elem = arr.get(*nth);

    ILitType::Result(match elem {
        Some(found) => res!(Ok, any => found.clone()),
        None => res!(Err, str_dy => format!("invalid index `{nth}`")),
    })
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
#[signature(args => string: string, char: string)]
pub fn split(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    let split = string
        .split(char.as_ref())
        .map(|s| ILitType::String(s.into()))
        .collect_into_rc_slice();

    ILitType::Array(split)
}

/// Split string into individual characters.
///
/// # Signature
/// ```
/// let ~internal chars(string: string): [string];
/// ```
///
/// # Example
/// ```
/// let my_string = "hello";
/// let split = chars(my_string);
/// for (part in split) {
///     printf("%s\n", [part]);
/// }
/// ```
///
/// ```text
/// h
/// e
/// l
/// l
/// o
/// ```
#[signature(args => string: string)]
pub fn chars(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Array(
        string
            .chars()
            .map(|s| ILitType::String(s.to_string().into()))
            .collect_into_rc_slice(),
    )
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
#[signature(args => probs_arr: any)]
pub fn len(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Integer(match probs_arr {
        ILitType::Array(arr) => arr.len(),
        ILitType::Integer(int) => *int,
        ILitType::String(str) => str.len(),
        ILitType::Unit => 0,
        ILitType::Result(_) => 1,
        ILitType::Stream(f) => match f {
            IStreamHandle::File(handle) => handle.borrow().metadata().unwrap().len() as usize,
            _ => unimplemented!("not yet kitten whiskers, daddy will discuss it later"),
        },
        ILitType::File(f) => f.as_os_str().len(),
    })
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
#[signature(args => arr: [any])]
pub fn enumerate(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Array(
        arr.iter()
            .enumerate()
            .map(|(idx, elem)| ILitType::Array(Rc::new([ILitType::Integer(idx), elem.clone()])))
            .collect_into_rc_slice(),
    )
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

/// Yield back only the first `up_to` elements of an array.
///
/// # Signature
/// ```
/// let ~internal only(arr: [any], up_to: integer): [any];
/// ```
///
/// # Details
/// Yields an empty array if `arr` is empty.
///
/// # Example
/// ```
/// let my_string = "Mary had a little lamb";
/// let split = split(my_string, " ");
/// let arr = only(split, 2);
/// for (i in arr) printf("%s\n", [i]);
/// ```
///
/// ```text
/// Mary
/// had
/// ```
#[signature(args => arr: [any], up_to: integer)]
pub fn only(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    let up_to = *up_to.min(&arr.len());

    ILitType::Array(Rc::from(&arr[..up_to]))
}

/// Reverse an array.
///
/// # Signature
/// ```
/// let ~internal rev(arr: [any]): [any];
/// ```
///
/// # Example
/// ```
/// let my_string = "Mary had a little lamb";
/// let split = split(my_string, " ");
/// let arr = reverse(split);
/// for (i in arr) printf("%s\n", [i]);
/// ```
///
/// ```text
/// lamb
/// little
/// a
/// had
/// Mary
/// ```
#[signature(args => arr: [any])]
pub fn rev(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Array(arr.iter().cloned().rev().collect_into_rc_slice())
}
