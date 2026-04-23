use std::{cell::RefCell, fs::File, path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub enum ILitType {
    Integer(usize),
    String(Rc<str>),
    Unit,
    Result(IResultBranch),
    Array(Box<[Self]>),
    Stream(IStreamHandle),
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub enum IResultBranch {
    Ok(Box<ILitType>),
    Err(Box<ILitType>),
}

#[derive(Debug, Clone)]
pub enum IStreamHandle {
    File(Rc<RefCell<File>>),
    Stdout,
    Stdin,
    Buffer(Vec<u8>),
}
