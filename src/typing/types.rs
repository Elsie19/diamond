#[derive(Debug, Clone)]
pub enum Type {
    String,
    Integer,
    Unit,
    Array(Box<Self>),
    Stream,
    File,
    Result(Box<Self>, Box<Self>),
}
