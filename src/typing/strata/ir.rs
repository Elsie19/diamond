use crate::typing::types::Type;

pub type VarId = usize;
pub type FuncId = usize;

pub enum StrataExpr {
    Integer(usize),
    String(String),
    Unit,
    Var(VarId),

    FuncCall {
        id: FuncId,
        args: Vec<Self>,
        ret: Type,
    },

    Let {
        id: VarId,
        ty: Type,
        value: Box<Self>,
        body: Box<Self>,
    },
}
