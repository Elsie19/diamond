use crate::parse::types::PType;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Type {
    String,
    Integer,
    #[default]
    Unit,
    Array(Box<Self>),
    Stream,
    File,
    Result(Box<Self>, Box<Self>),
}

impl From<PType<'_>> for Type {
    fn from(value: PType<'_>) -> Self {
        match value {
            PType::Unit(_) => Self::Unit,
            PType::File(_) => Self::File,
            PType::Array(ty) => Self::Array(Box::new(Self::from(*ty.into_inner()))),
            PType::Stream(_) => Self::Stream,
            PType::String(_) => Self::String,
            PType::Result(re) => {
                let (a, b) = {
                    let re = re.into_inner();
                    let a = Self::from(*re.0);
                    let b = Self::from(*re.1);
                    (Box::new(a), Box::new(b))
                };
                Self::Result(a, b)
            }
        }
    }
}
