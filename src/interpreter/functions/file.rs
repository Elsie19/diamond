use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
};

use crate::interpreter::{
    engine::Engine,
    types::{ILitType, IResultBranch, IStreamHandle},
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

    if let ILitType::String(path) = arg {
        Some(ILitType::File(PathBuf::from(path.as_ref())))
    } else {
        None
    }
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

    if let ILitType::File(path) = arg {
        match File::create(path) {
            Ok(_) => Some(ILitType::Result(IResultBranch::Ok(Box::new(
                ILitType::File(path.to_path_buf()),
            )))),
            Err(err) => Some(ILitType::Result(IResultBranch::Err(Box::new(
                ILitType::String(err.to_string().into()),
            )))),
        }
    } else {
        None
    }
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

    if let ILitType::File(path) = arg {
        match OpenOptions::new()
            .read(true)
            .append(true)
            .create(false)
            .open(path)
        {
            Ok(stream) => Some(ILitType::Result(IResultBranch::Ok(Box::new(
                ILitType::Stream(IStreamHandle::File(Rc::new(RefCell::new(stream)))),
            )))),
            Err(err) => Some(ILitType::Result(IResultBranch::Err(Box::new(
                ILitType::String(err.to_string().into()),
            )))),
        }
    } else {
        None
    }
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

    match stream {
        IStreamHandle::File(file) => match file.borrow_mut().write_all(contents.as_bytes()) {
            Ok(_) => Some(ILitType::Result(IResultBranch::Ok(Box::new(
                ILitType::Unit,
            )))),
            Err(err) => Some(ILitType::Result(IResultBranch::Err(Box::new(
                ILitType::String(err.to_string().into()),
            )))),
        },
        _ => todo!("haven't done shit yet"),
    }
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

    match stream {
        IStreamHandle::File(handle) => match handle.borrow_mut().read_to_string(&mut contents) {
            Ok(_) => Some(ILitType::Result(IResultBranch::Ok(Box::new(
                ILitType::Array(
                    contents
                        .lines()
                        .map(|s| ILitType::String(s.into()))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
            )))),
            Err(e) => Some(ILitType::Result(IResultBranch::Err(Box::new(
                ILitType::String(e.to_string().into()),
            )))),
        },
        _ => todo!("not done yet"),
    }
}
