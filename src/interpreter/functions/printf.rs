use sig_macro::signature;

use crate::interpreter::{engine::Engine, types::ILitType};
use std::fmt::Write;

/// Prints out unformatted text.
///
/// # Signature
/// ```
/// let ~internal puts(format: string): unit;
/// ```
///
/// # Example
/// ```
/// puts("hello, console!");
/// ```
#[signature(args => format: string)]
pub fn puts(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    print!("{format}");

    Some(ILitType::Unit)
}

/// Prints out unformatted text.
///
/// # Signature
/// ```
/// let ~internal printf(format: string, args: [any]): integer;
/// ```
///
/// # Details
/// The following format specifiers are allowed:
///
/// | Format Specifier |     Type    |                    What it does                    |
/// |:----------------:|:-----------:|:--------------------------------------------------:|
/// | `%s`             |   `string`  |                                 Formats the string |
/// | `%s`             | other types | Formats the string acccording to rust debug format |
/// | `%d`/`%u`        |  `integer`  |                                Formats the integer |
/// | `%f`             | `integer`   |                     Formats the integer as a float |
/// | `%a`             | `[any]`     |          Formats the array as it would be assigned |
/// | other            | `any`       |                 Formats the argument as `%{value}` |
///
/// Because Diamond does not have variadic arguments, `printf` uses an array a holder for
/// arguments, not unlike [Zig](https://ziglang.org/documentation/master/#Hello-World:~:text=s%7D!%5Cn%22%2C-,.%7B%22World%22%7D,-\)%3B%0A%7D).
///
/// `printf` will return the length of the string printed to the console.
///
/// # Example
/// ```
/// printf("%s, %s!\n", ["Hello", "World"]);
/// ```
pub fn printf(engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let ret = sprintf(engine, args);
    let Some(ILitType::String(s)) = ret else {
        unreachable!("oopsies");
    };

    print!("{s}");

    Some(ILitType::Integer(s.len()))
}

/// Returns formatted text.
///
/// # Signature
/// ```
/// let ~internal sprintf(format: string, args: [any]): string;
/// ```
///
/// # Details
/// See [`printf`] for more information.
///
/// # Example
/// ```
/// let formatted = sprintf("%s, %s!\n", ["Hello", "World"]);
/// ```
pub fn sprintf(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let (fmt, args) = match args {
        [fmt] => (fmt, None),
        [fmt, args @ ..] => (fmt, Some(args)),
        [] => unreachable!("how did we get here?"),
    };

    let fmt = fmt.as_string_rep();

    let ILitType::Array(args) = &args.unwrap()[0] else {
        unreachable!("type check failed. Make sure you are passing an array.")
    };

    // We know at least this much is true.
    let mut buf = String::with_capacity(fmt.len());

    let mut cur_arg = 0;

    let mut it = fmt.chars().peekable();
    while let Some(char) = it.next() {
        if char == '\\' {
            match it.next() {
                Some('n') => buf.push('\n'),
                Some('t') => buf.push('\t'),
                Some('\\') | None => buf.push('\\'),
                Some(other) => {
                    buf.push('\\');
                    buf.push(other);
                }
            }
            continue;
        }

        if char != '%' {
            buf.push(char);
            continue;
        }

        if let Some('%') = it.peek() {
            it.next();
            buf.push('%');
            continue;
        }

        let Some(spec) = it.next() else {
            buf.push('%');
            break;
        };

        let arg = args.get(cur_arg).unwrap_or(&ILitType::Unit);

        let out = match (spec, arg) {
            ('s', any) => any.as_string_rep(),
            ('d' | 'u', ILitType::Integer(i)) => i.to_string().into(),
            #[allow(clippy::cast_precision_loss)]
            ('f', ILitType::Integer(i)) => (*i as f64).to_string().into(),
            ('a', ILitType::Array(a)) => {
                let mut mini_buf = String::from("[");

                for (idx, v) in a.iter().enumerate() {
                    if idx > 0 {
                        mini_buf.push_str(", ");
                    }
                    let _ = write!(buf, "{v:?}");
                }

                mini_buf.push(']');
                mini_buf.into()
            }
            (unknown, _) => format!("%{unknown}").into(),
        };

        buf.push_str(&out);
        cur_arg += 1;
    }

    Some(ILitType::String(buf.into()))
}
