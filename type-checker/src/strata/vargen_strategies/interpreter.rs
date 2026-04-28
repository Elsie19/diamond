use crate::strata::VarGenerator;

#[derive(Debug, Default)]
pub struct VarGenInterpreter {
    id: usize,
}

impl VarGenerator for VarGenInterpreter {
    fn init() -> Self {
        Self::default()
    }

    fn fresh(&mut self) -> usize {
        let num = self.id;
        self.id += 1;
        num
    }
}
