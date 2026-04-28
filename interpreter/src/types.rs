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
                // Since nothing is zero length when printed, we can guess the absolute lowest
                // bounds that the string will have. Obviously it will have more but if we can defer
                // reallocation that's always good.
                let mut mini_buf = String::with_capacity(a.len());
                mini_buf.push('[');

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
                    let ptr: *const _ = &raw const file_handle;
                    Cow::Owned(format!("{ptr:p}"))
                }
                IStreamHandle::Stdout => todo!(),
                IStreamHandle::Stdin => todo!(),
                IStreamHandle::Buffer(_items) => todo!(),
            },
            ILitType::File(path) => path.as_os_str().to_string_lossy(),
        }
    }

    pub fn string<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self::String(s.into().into())
    }
}
