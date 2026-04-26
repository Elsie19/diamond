use std::{borrow::Cow, cell::RefCell, fmt::Write, fs::File, path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub enum ILitType {
    Integer(usize),
    String(Rc<str>),
    Unit,
    Result(IResultBranch),
    Array(Rc<[Self]>),
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

impl ILitType {
    pub fn as_string_rep(&self) -> Cow<'_, str> {
        match self {
            ILitType::Integer(i) => Cow::Owned(i.to_string()),
            ILitType::String(s) => Cow::Borrowed(s),
            ILitType::Unit => Cow::Borrowed("()"),
            ILitType::Result(iresult_branch) => match iresult_branch {
                IResultBranch::Ok(ok) => ok.as_string_rep(),
                IResultBranch::Err(err) => err.as_string_rep(),
            },
            ILitType::Array(a) => {
                let mut mini_buf = String::from("[");

                for (idx, v) in a.iter().enumerate() {
                    if idx > 0 {
                        mini_buf.push_str(", ");
                    }
                    let _ = write!(mini_buf, "{v:?}");
                }

                mini_buf.push(']');
                mini_buf.into()
            }
            ILitType::Stream(handle) => match handle {
                IStreamHandle::File(file_handle) => {
                    let ptr: *const _ = &file_handle;
                    Cow::Owned(format!("{:p}", ptr))
                }
                IStreamHandle::Stdout => todo!(),
                IStreamHandle::Stdin => todo!(),
                IStreamHandle::Buffer(_items) => todo!(),
            },
            ILitType::File(path) => path.as_os_str().to_string_lossy(),
        }
    }
}
