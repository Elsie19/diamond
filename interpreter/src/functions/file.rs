use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use sig_macro::signature;

use crate::{
    engine::Engine,
    res,
    types::{ILitType, IStreamHandle},
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
#[signature(args => path: string)]
pub fn file(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::File(PathBuf::from(path.as_ref()))
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
#[signature(args => path: file)]
pub fn create(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(match File::create(path) {
        Ok(_) => res!(Ok, file => path.clone()),
        Err(err) => res!(Err, str_dy => err.to_string()),
    })
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
#[signature(args => path: file)]
pub fn open(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(
        match OpenOptions::new()
            .read(true)
            .append(true)
            .create(false)
            .open(path)
        {
            Ok(stream) => res!(Ok, stream => stream),
            Err(err) => res!(Err, str_dy => err.to_string()),
        },
    )
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
#[signature(args => stream: stream, contents: string)]
pub fn dump(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(match stream {
        IStreamHandle::File(file) => match file.borrow_mut().write_all(contents.as_bytes()) {
            Ok(()) => res!(Ok, unit),
            Err(err) => res!(Err, str_dy => err.to_string()),
        },
        _ => todo!("haven't done shit yet"),
    })
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
#[signature(args => stream: stream)]
pub fn lines(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    ILitType::Result(match stream {
        IStreamHandle::File(handle) => {
            let mut file = handle.borrow_mut();
            let size = file.metadata().unwrap().len();
            let mut contents = String::with_capacity(size as usize);
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    res!(Ok, arr => contents.lines().map(|s| ILitType::String(s.into())).collect::<Vec<_>>())
                }
                Err(e) => res!(Err, str_dy => e.to_string()),
            }
        }
        _ => todo!("not done yet"),
    })
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
#[signature(args => path: file)]
pub fn fpop(_engine: &mut Engine<'_>, args: &[ILitType]) -> ILitType {
    let mut path = path.clone();
    ILitType::Result(if path.pop() {
        res!(Ok, file => path)
    } else {
        res!(Err, str => "file has no parent")
    })
}
