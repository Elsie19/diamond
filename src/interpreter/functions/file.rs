use std::{fs::File, path::PathBuf, rc::Rc};

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
        match File::open(path) {
            Ok(stream) => Some(ILitType::Result(IResultBranch::Ok(Box::new(
                ILitType::Stream(IStreamHandle::File(Rc::new(stream))),
            )))),
            Err(err) => Some(ILitType::Result(IResultBranch::Err(Box::new(
                ILitType::String(err.to_string()),
            )))),
        }
    } else {
        None
    }
}
