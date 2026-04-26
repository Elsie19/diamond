use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    rc::Rc,
};

use crate::{
    interpreter::{
        engine::Engine,
        types::{ILitType, IResultBranch, IStreamHandle},
    },
    res,
};

/// File type wrapper.
///
/// # Signature
/// ```
/// let ~internal file(path: string): file;
/// ```
///
/// # Details
/// Creates a file type from a string path.
///
/// # Example
/// ```
/// let file = file("some_file.txt");
/// ```
pub fn file(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    let ILitType::String(path) = arg else {
        unreachable!("type checked");
    };

    Some(ILitType::File(PathBuf::from(path.as_ref())))
}

/// Create a file.
///
/// # Signature
/// ```
/// let ~internal create(path: file): result(file, string);
/// ```
///
/// # Details
/// Returns the file path if successfully created, or an error if not.
///
/// # Example
/// ```
/// let file = file("some_file.txt");
/// let created = create(file)!;
/// ```
pub fn create(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    let ILitType::File(path) = arg else {
        unreachable!("type checked");
    };

    Some(ILitType::Result(match File::create(path) {
        Ok(_) => res!(Ok, file => path.to_path_buf()),
        Err(err) => res!(Err, str_dy => err.to_string()),
    }))
}

/// Open a file.
///
/// # Signature
/// ```
/// let ~internal open(path: file): result(stream, string);
/// ```
///
/// # Details
/// Returns the stream if successfully created, or an error if not.
///
/// # Example
/// ```
/// let file = file("some_file.txt");
/// let created = open(file)!;
/// let stream = create(created)!;
/// ```
pub fn open(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    let ILitType::File(path) = arg else {
        unreachable!("type checked");
    };

    Some(ILitType::Result(
        match OpenOptions::new()
            .read(true)
            .append(true)
            .create(false)
            .open(path)
        {
            Ok(stream) => res!(Ok, stream => stream),
            Err(err) => res!(Err, str_dy => err.to_string()),
        },
    ))
}

/// Dump text to a stream.
///
/// # Signature
/// ```
/// let ~internal dump(stream: stream, contents: string): result(unit, string);
/// ```
///
/// # Details
/// Returns an error if it could not write to the stream.
///
/// # Example
/// ```
/// let file = file("some_file.txt");
/// let created = open(file)!;
/// let stream = create(created)!;
/// dump(stream, "here is the text inside `some_file.txt`")!;
/// ```
pub fn dump(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Stream(stream), ILitType::String(contents)] = args else {
        unreachable!("type checked");
    };

    Some(ILitType::Result(match stream {
        IStreamHandle::File(file) => match file.borrow_mut().write_all(contents.as_bytes()) {
            Ok(_) => res!(Ok, unit),
            Err(err) => res!(Err, str_dy => err.to_string()),
        },
        _ => todo!("haven't done shit yet"),
    }))
}

/// Get lines of stream.
///
/// # Signature
/// ```
/// let ~internal lines(stream: stream): result([string], string);
/// ```
///
/// # Details
/// Returns lines on success, or error on failure.
///
/// # Example
/// ```
/// let stream = open(file("newline_list.txt"))!;
/// for (i in lines(stream)!) {
///     printf("%s\n", [i]);
/// };
/// ```
pub fn lines(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::Stream(stream)] = args else {
        unreachable!("type checked");
    };

    let mut contents = String::new();

    Some(ILitType::Result(match stream {
        IStreamHandle::File(handle) => match handle.borrow_mut().read_to_string(&mut contents) {
            Ok(_) => {
                res!(Ok, arr => contents.lines().map(|s| ILitType::String(s.into())).collect::<Vec<_>>())
            }
            Err(e) => res!(Err, str_dy => e.to_string()),
        },
        _ => todo!("not done yet"),
    }))
}

/// Skip `n` amount of lines in stream.
///
/// # Signature
/// ```
/// let ~internal skip(stream: stream, n: integer): result([string], string);
/// ```
///
/// # Details
/// Returns lines on success, or error on failure.
///
/// # Example
/// ```
/// let stream = open(file("people.csv"))!;
/// for (i in skip(stream, 1)!) {
///     printf("%s\n", [i]);
/// };
/// ```
///
/// ```csv
/// Ainsley,5-29-05,female
/// Sam,10-21-07,male
/// ```
pub fn skip(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 2);

    let [ILitType::Stream(stream), ILitType::Integer(skip)] = args else {
        unreachable!("type checked");
    };

    match stream {
        IStreamHandle::File(handle) => {
            let file = &*handle.borrow_mut();
            let buf = BufReader::new(file);
            let lines = buf
                .lines()
                .skip(*skip)
                .map(|line| line.map(|s| ILitType::String(s.into())))
                .collect::<Result<Vec<_>, _>>();
            Some(ILitType::Result(match lines {
                Ok(lines) => res!(Ok, arr => lines),
                Err(err) => res!(Err, str_dy => err.to_string()),
            }))
        }
        _ => todo!("not done yet"),
    }
}

/// Pop last path from path.
///
/// # Signature
/// ```
/// let ~internal fpop(path: file): result(file, string);
/// ```
///
/// # Details
/// Returns truncated file on success, or error if the path has no parent.
///
/// # Example
/// ```
/// let popped = fpop(file("path/to/thing.csv"))!;
/// printf("%s\n", [popped]);
/// ```
///
/// ```text
/// path/to
/// ```
pub fn fpop(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);

    let [ILitType::File(file)] = args else {
        unreachable!("type checked");
    };

    let mut path = file.clone();
    Some(ILitType::Result(match path.pop() {
        true => res!(Ok, file => path),
        false => res!(Err, str => "file has no parent"),
    }))
}
