use std::{path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub enum ILitType {
    Integer(usize),
    String(String),
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
    File(Rc<std::fs::File>),
    Stdout,
    Stdin,
    Buffer(Vec<u8>),
}
