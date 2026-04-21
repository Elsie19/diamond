use std::collections::HashMap;

use crate::{
    interpreter::types::ILitType,
    typing::{pass_one::FuncTable, strata::IR},
};

#[derive(Debug)]
pub struct Engine<'a> {
    ir: &'a [IR],
    funcs: &'a FuncTable<'a>,
    vars: HashMap<String, ILitType>,
}

impl<'a> Engine<'a> {
    pub fn new(ir: &'a [IR], funcs: &'a FuncTable<'a>) -> Self {
        Self {
            ir,
            funcs,
            vars: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        for node in self.ir {
            self.eval(node);
        }
    }

    fn eval(&mut self, node: &IR) -> Option<ILitType> {
        match node {
            IR::FuncLet {
                name,
                args,
                internal: _,
                ret,
                body,
            } => todo!("funclet"),
            IR::Grouping {
                inner,
                expr_end,
                redirect,
            } => {
                if let Some((redir_ir, bind)) = redirect {
                    let val = self
                        .eval(redir_ir)
                        .expect("redirect did not produce value!!!");
                    self.vars.insert(bind.clone(), val);
                }

                for node in inner {
                    self.eval(node);
                }

                if let Some(expr) = expr_end {
                    self.eval(expr)
                } else {
                    Some(ILitType::Unit)
                }
            }
            IR::For { bind, iter, body } => self.eval_for_loop(bind, iter, body),
            IR::Let { name, ty: _, value } => {
                debug_assert_eq!(value.len(), 1);
                let val = self.eval(&value[0]).expect("did not produce value!!!");
                self.vars.insert(name.clone(), val)
            }
            IR::Match { expr, arms } => todo!("match"),
            IR::FuncCall { name, args, unwrap } => todo!("funccall"),
            IR::Integer(i) => Some(ILitType::Integer(*i)),
            IR::String(s) => Some(ILitType::String(s.to_string())),
            IR::Ident(ident) => self.vars.get(ident).cloned(),
            IR::Array(irs) => {
                let mut elems = Vec::with_capacity(irs.len());
                for elem in irs {
                    let val = self
                        .eval(elem)
                        .expect("array element didn't provide a value");
                    elems.push(val);
                }
                Some(ILitType::Array(elems.into_boxed_slice()))
            }
            IR::Unit => Some(ILitType::Unit),
            IR::Result { ok, err } => todo!("result"),
            IR::Expr(ir) => todo!("expr"),
            IR::Stmt(ir) => todo!("stmt"),
        }
    }

    fn eval_for_loop(&mut self, bind: &str, iter: &[IR], body: &[IR]) -> Option<ILitType> {
        debug_assert_eq!(iter.len(), 1);
        let iter = &iter[0];
        let iter = self.eval(iter).expect("iter did not produce value");
        let ILitType::Array(iter) = iter else {
            unreachable!("arrays are the only iterable thing");
        };

        let mut last = None;

        for rust_idx in iter {
            self.vars.insert(bind.to_string(), rust_idx);
            for ir in body {
                last = self.eval(ir);
            }
        }

        last
    }
}
