use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    rc::Rc,
};

use crate::interpreter::{
    engine::Engine,
    types::{ILitType, IResultBranch, IStreamHandle},
};

pub fn file(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    if let ILitType::String(path) = arg {
        Some(ILitType::File(PathBuf::from(path)))
    } else {
        None
    }
}

pub fn create(_engine: &mut Engine<'_>, args: &[ILitType]) -> Option<ILitType> {
    debug_assert_eq!(args.len(), 1);
    let arg = &args[0];

    if let ILitType::File(path) = arg {
        match File::create(path) {
            Ok(_) => Some(ILitType::Result(IResultBranch::Ok(Box::new(
                ILitType::File(path.to_path_buf()),
            )))),
            Err(err) => Some(ILitType::Result(IResultBranch::Err(Box::new(
                ILitType::String(err.to_string()),
            )))),
        }
    } else {
        None
    }
}

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
                ILitType::String(err.to_string()),
            )))),
        }
    } else {
        None
    }
}

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
                ILitType::String(err.to_string()),
            )))),
        },
        _ => todo!("haven't done shit yet"),
    }
}
