use std::rc::Rc;

use collect_into_rc_slice::CollectIntoRcSlice;
use type_checker::strata::{IR, IRMatchArm};

use shared::unreachable_unchecked;

use crate::{
    stdlib::{Functions, RuntimeFunc, UserFunc},
    types::{ILitType, IResultBranch},
};

type Val = ILitType;

#[derive(Debug)]
pub struct Engine<'a> {
    frames: Vec<ILitType>,
    funcs: Functions<'a>,
    argv: Rc<[ILitType]>,
}

impl Engine<'_> {
    pub fn new<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self::new_precomp(args, 0)
    }

    pub fn new_precomp<I, T>(args: I, size: usize) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            frames: vec![ILitType::Unit; size],
            funcs: Functions::stdlib(),
            argv: args
                .into_iter()
                .map(|s| ILitType::String(s.into().into()))
                .collect_into_rc_slice(),
        }
    }

    pub fn args(&self) -> &Rc<[ILitType]> {
        &self.argv
    }

    fn get_var(&self, name: usize) -> &ILitType {
        &self.frames[name]
    }

    fn set_var(&mut self, name: usize, val: ILitType) {
        self.frames[name] = val;
    }

    pub fn run<I>(&mut self, ir: I)
    where
        I: IntoIterator<Item = IR>,
    {
        for node in ir {
            self.eval(node);
        }
    }

    fn eval(&mut self, node: IR) -> Val {
        match node {
            IR::FuncLet {
                name,
                args,
                internal: _,
                ret,
                body,
            } => {
                let func = RuntimeFunc::User(UserFunc {
                    args: args.into_boxed_slice(),
                    body: *body,
                    ret,
                });

                self.funcs.insert(name.to_string(), func);

                ILitType::Unit
            }
            IR::Grouping { inner, redirect } => self.eval_grouping(inner, redirect),
            IR::For { bind, iter, body } => self.eval_for_loop(bind, *iter, *body),
            IR::Let { name, ty: _, value } => {
                let val = self.eval(*value);
                self.set_var(name, val.clone());
                val
            }
            IR::Match { expr, arms } => self.eval_match(*expr, arms),
            IR::FuncCall { name, args, unwrap } => self.eval_funccall(&name, args, unwrap),
            IR::Integer(i) => ILitType::Integer(i),
            IR::String(s) => ILitType::String(s),
            IR::Ident(ident) => self.get_var(ident).clone(),
            IR::Array(irs) => {
                let elems = irs
                    .into_iter()
                    .map(|x| self.eval(x))
                    .collect_into_rc_slice();
                ILitType::Array(elems)
            }
            IR::Unit => ILitType::Unit,
            IR::Result { ok: _, err: _ } => todo!("result"),
            IR::Expr(ir) => self.eval(*ir),
            IR::Stmt(ir) => {
                self.eval(*ir);
                ILitType::Unit
            }
        }
    }

    fn eval_funccall<I>(&mut self, name: &str, args: I, unwrap: bool) -> Val
    where
        I: IntoIterator<Item = IR>,
    {
        let func = match self.funcs.resolve(name) {
            Some(func_name) => func_name.clone(),
            None => panic!("function `{name}` not found! Did you add the internal function yet?"),
        };

        let evaled_args = args
            .into_iter()
            .map(|x| self.eval(x))
            .collect_into_rc_slice();

        let ret = match func {
            RuntimeFunc::Internal(f) => f(self, &evaled_args),
            RuntimeFunc::User(user_func) => {
                let func_args = user_func.args;

                for (i, (arg_name, _)) in func_args.iter().enumerate() {
                    let val = evaled_args.get(i).cloned().unwrap_or(ILitType::Unit);

                    self.set_var(*arg_name, val);
                }

                let last = self.eval(user_func.body);

                last
            }
        };

        if unwrap {
            // SAFETY: If we've gotten this far, the type-checker has proven that any time `unwrap`
            // is true, the type alongside must be [`ILitType::Result`].
            let ILitType::Result(iresult_branch) = ret else {
                unreachable_unchecked!()
            };

            match iresult_branch {
                IResultBranch::Ok(ilit_type) => *ilit_type,
                IResultBranch::Err(ilit_type) => {
                    use crate::functions::system::panic as internal_panic;
                    internal_panic(self, &[*ilit_type, ILitType::Array(Rc::new([]))]);
                    unreachable_unchecked!()
                }
            }
        } else {
            ret
        }
    }

    fn eval_match<I>(&mut self, expr: IR, arms: I) -> Val
    where
        I: IntoIterator<Item = IRMatchArm>,
    {
        let ILitType::Result(result) = self.eval(expr) else {
            unreachable_unchecked!()
        };

        for arm in arms {
            let IRMatchArm { bind, is_ok, body } = arm;

            let active = match (&result, is_ok) {
                (IResultBranch::Ok(v), true) | (IResultBranch::Err(v), false) => Some(v.clone()),
                _ => None,
            };

            if let Some(val) = active {
                self.set_var(bind, *val);

                let last = self.eval(*body);

                return last;
            }
        }

        unreachable_unchecked!()
    }

    fn eval_for_loop(&mut self, bind: usize, iter: IR, body: IR) -> Val {
        let ILitType::Array(iter) = self.eval(iter) else {
            unreachable_unchecked!()
        };

        let mut last = ILitType::Unit;

        for rust_idx in iter.iter() {
            self.set_var(bind, rust_idx.clone());
            last = self.eval(body.clone());
        }

        last
    }

    fn eval_grouping<I>(&mut self, inner: I, redirect: Option<(Box<IR>, usize)>) -> Val
    where
        I: IntoIterator<Item = IR>,
    {
        if let Some((redir_ir, bind)) = redirect {
            let val = self.eval(*redir_ir);
            self.set_var(bind, val);
        }

        let mut last_val = ILitType::Unit;

        for node in inner {
            last_val = self.eval(node);
        }

        last_val
    }
}
