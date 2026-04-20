use crate::typing::strata::VarGenerator;

#[derive(Debug, Default)]
pub struct QbeVarGen {
    ct: usize,
}

impl VarGenerator for QbeVarGen {
    fn init() -> Self {
        Self::default()
    }

    fn fresh<S>(&mut self, _str: S) -> String
    where
        S: AsRef<str>,
    {
        let ct = self.ct;
        let ret = format!(".{}", ct);
        self.ct += 1;
        ret
    }
}
