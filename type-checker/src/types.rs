use std::borrow::Cow;

use bincode::{Decode, Encode};
use enum_as_inner::EnumAsInner;
use parse::types::PType;

#[derive(Debug, Clone, Default, EnumAsInner, Decode, Encode)]
pub enum Type {
    String,
    Integer,
    #[default]
    Unit,
    Array(Box<Self>),
    Stream,
    File,
    Result(Box<Self>, Box<Self>),
    Unret,
    Any,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        use Type as S;
        match (self, other) {
            // This allows unret types (panicking and stuff) to always work in places where it
            // expects some other type.
            (S::Unret, _) | (_, S::Unret) | (S::Any, _) | (_, S::Any) => true,

            (S::String, S::String) => true,
            (S::Integer, S::Integer) => true,
            (S::Unit, S::Unit) => true,
            (S::Stream, S::Stream) => true,
            (S::File, S::File) => true,

            (S::Array(a), S::Array(b)) => a == b,
            (S::Result(a1, b1), S::Result(a2, b2)) => a1 == a2 && b1 == b2,

            _ => false,
        }
    }
}

impl Type {
    pub fn is_any_array(&self) -> bool {
        match self {
            Self::Array(inner) => matches!(&**inner, Self::Any),
            _ => false,
        }
    }

    pub fn as_display_ty(&self) -> Cow<'_, str> {
        match self {
            Type::String => Cow::Borrowed("string"),
            Type::Integer => Cow::Borrowed("integer"),
            Type::Unit => Cow::Borrowed("unit"),
            Type::Array(ty) => Cow::Owned(format!("[{}]", ty.as_display_ty())),
            Type::Stream => Cow::Borrowed("stream"),
            Type::File => Cow::Borrowed("file"),
            Type::Unret => Cow::Borrowed("unret"),
            Type::Any => Cow::Borrowed("any"),
            Type::Result(ok, err) => {
                format!("result({}, {})", ok.as_display_ty(), err.as_display_ty()).into()
            }
        }
    }
}

impl From<PType<'_>> for Type {
    fn from(value: PType<'_>) -> Self {
        match value {
            PType::Unit(_) => Self::Unit,
            PType::Integer(_) => Self::Integer,
            PType::File(_) => Self::File,
            PType::Array(ty) => Self::Array(Box::new(Self::from(*ty.into_inner()))),
            PType::Stream(_) => Self::Stream,
            PType::String(_) => Self::String,
            PType::Unret(_) => Self::Unret,
            PType::Any(_) => Self::Any,
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
