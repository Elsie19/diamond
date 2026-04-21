use crate::interpreter::{engine::Engine, types::ILitType};

pub fn puts<'a>(_engine: &mut Engine<'a>, args: &[ILitType]) -> Option<ILitType> {
    let ILitType::String(s) = &args[0] else {
        unreachable!("oopsies");
    };

    print!("{s}");

    Some(ILitType::Unit)
}

pub fn printf<'a>(engine: &mut Engine<'a>, args: &[ILitType]) -> Option<ILitType> {
    let ret = sprintf(engine, args);
    let Some(ILitType::String(s)) = ret else {
        unreachable!("oopsies");
    };

    print!("{}", s);

    Some(ILitType::Integer(s.len()))
}

pub fn sprintf<'a>(_engine: &mut Engine<'a>, args: &[ILitType]) -> Option<ILitType> {
    let (fmt, args) = match args {
        [fmt] => (fmt, None),
        [fmt, args @ ..] => (fmt, Some(args)),
        [] => unreachable!("how did we get here?"),
    };

    let ILitType::String(fmt) = fmt else {
        unreachable!("checked at type checking");
    };

    let args = match args {
        Some(a) => match &a[0] {
            ILitType::Array(arr) => arr,
            _ => unreachable!("type checked"),
        },
        None => return Some(ILitType::String(fmt.to_string())),
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
                Some('\\') => buf.push('\\'),
                Some(other) => {
                    buf.push('\\');
                    buf.push(other);
                }
                None => buf.push('\\'),
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

        let spec = match it.next() {
            Some(s) => s,
            None => {
                buf.push('%');
                break;
            }
        };

        let arg = args.get(cur_arg).unwrap_or(&ILitType::Unit);

        let out = match (spec, arg) {
            ('s', ILitType::String(s)) => s.clone(),
            ('s', other) => format!("{other:?}"),
            ('d' | 'u', ILitType::Integer(i)) => i.to_string(),
            ('f', ILitType::Integer(i)) => (*i as f64).to_string(),
            ('a', ILitType::Array(a)) => {
                let mut mini_buf = String::from("[");

                for (idx, v) in a.iter().enumerate() {
                    if idx > 0 {
                        buf.push_str(", ");
                    }
                    buf.push_str(&format!("{:?}", v));
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
