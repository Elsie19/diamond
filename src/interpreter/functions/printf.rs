use crate::interpreter::{engine::Engine, types::ILitType};
use std::fmt::Write;

pub fn puts(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let ILitType::String(s) = &args[0] else {
        unreachable!("oopsies");
    };

    print!("{s}");

    Some(ILitType::Unit)
}

pub fn printf(engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let ret = sprintf(engine, args);
    let Some(ILitType::String(s)) = ret else {
        unreachable!("oopsies");
    };

    print!("{s}");

    Some(ILitType::Integer(s.len()))
}

pub fn sprintf(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    let (fmt, args) = match args {
        [fmt] => (fmt, None),
        [fmt, args @ ..] => (fmt, Some(args)),
        [] => unreachable!("how did we get here?"),
    };

    let ILitType::String(fmt) = fmt else {
        unreachable!("checked at type checking");
    };

    let ILitType::Array(args) = &args.unwrap()[0] else {
        unreachable!("type checked")
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
            ('s', ILitType::String(s)) => s.clone(),
            ('s', other) => format!("{other:?}"),
            ('d' | 'u', ILitType::Integer(i)) => i.to_string(),
            #[allow(clippy::cast_precision_loss)]
            ('f', ILitType::Integer(i)) => (*i as f64).to_string(),
            ('a', ILitType::Array(a)) => {
                let mut mini_buf = String::from("[");

                for (idx, v) in a.iter().enumerate() {
                    if idx > 0 {
                        buf.push_str(", ");
                    }
                    let _ = write!(buf, "{v:?}");
                }

                mini_buf.push(']');
                mini_buf
            }
            (unknown, _) => format!("%{unknown}"),
        };

        buf.push_str(&out);
        cur_arg += 1;
    }

    Some(ILitType::String(buf))
}
