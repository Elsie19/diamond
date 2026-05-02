use std::{collections::HashMap, rc::Rc};

use type_checker::strata::{IR, IRMatchArm};

use crate::{
    stdlib::{Functions, RuntimeFunc, UserFunc},
    types::{ILitType, IResultBranch},
};

type Val = ILitType;

#[derive(Debug)]
pub struct Engine<'a> {
    // Even though the IR generator ensures that all variables have unique identifiers, we still
    // need to have stack frames for recursion in function calls.
    frames: Vec<StackFrame>,
    funcs: Functions<'a>,
    argv: Rc<[ILitType]>,
}

#[derive(Debug, Clone, Default)]
pub struct StackFrame {
    vars: HashMap<usize, ILitType>,
}

impl<'a> Engine<'a> {
    pub fn new<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            frames: vec![StackFrame::default()],
            funcs: Functions::stdlib(),
            argv: args
                .into_iter()
                .map(|s| ILitType::String(s.into().into()))
                .collect::<Vec<_>>()
                .into(),
        }
    }

    pub fn args(&self) -> &Rc<[ILitType]> {
        &self.argv
    }

    fn get_var(&self, name: usize) -> Option<&ILitType> {
        self.frames.iter().rev().find_map(|f| f.vars.get(&name))
    }

    fn set_var(&mut self, name: usize, val: ILitType) {
        self.frames
            .last_mut()
            .expect("popped top frame, ruh roh")
            .vars
            .insert(name, val);
    }

    fn push_frame(&mut self) {
        self.frames.push(StackFrame::default());
    }

    fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub fn run<I>(&mut self, ir: I)
    where
        I: IntoIterator<Item = IR>,
    {
        for node in ir.into_iter() {
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
            // SAFETY: If a variable cannot be found here, it somehow got past both type-checking
            // and vargen, and I fucked up really badly.
            IR::Ident(ident) => unsafe { self.get_var(ident).cloned().unwrap_unchecked() },
            IR::Array(irs) => {
                let elems = irs.into_iter().map(|x| self.eval(x)).collect::<Vec<_>>();
                ILitType::Array(elems.into())
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
        let func = self.funcs.resolve(name).cloned().unwrap_or_else(|| {
            panic!("function `{name}` not found! Did you add the internal function yet?")
        });

        let evaled_args = args.into_iter().map(|x| self.eval(x)).collect::<Vec<_>>();

        let ret = match func {
            RuntimeFunc::Internal(f) => f(self, &evaled_args),
            RuntimeFunc::User(func) => {
                self.push_frame();

                for (i, (arg_name, _)) in func.args.iter().enumerate() {
                    let val = evaled_args.get(i).cloned().unwrap_or(ILitType::Unit);

                    self.set_var(*arg_name, val);
                }

                let last = self.eval(func.body);

                self.pop_frame();

                last
            }
        };

        if unwrap {
            match ret {
                ILitType::Result(iresult_branch) => match iresult_branch {
                    IResultBranch::Ok(ilit_type) => *ilit_type,
                    IResultBranch::Err(ilit_type) => {
                        use crate::functions::system::panic as internal_panic;
                        internal_panic(self, &[*ilit_type, ILitType::Array(Rc::new([]))]);
                        unreachable!("panicked above???");
                    }
                },
                err => panic!("expected `result`, but got `{err:?}`"),
            }
        } else {
            ret
        }
    }

    fn eval_match<I>(&mut self, expr: IR, arms: I) -> Val
    where
        I: IntoIterator<Item = IRMatchArm>,
    {
        let expr = self.eval(expr);

        let ILitType::Result(result) = expr else {
            unreachable!("type checked");
        };

        for arm in arms {
            let IRMatchArm { bind, is_ok, body } = arm;

            let active = match (&result, is_ok) {
                (IResultBranch::Ok(v), true) | (IResultBranch::Err(v), false) => Some(v.clone()),
                _ => None,
            };

            if let Some(val) = active {
                self.push_frame();
                self.set_var(bind, *val);

                let last = self.eval(*body);

                self.pop_frame();
                return last;
            }
        }

        unreachable!("match didn't find an arm");
    }

    fn eval_for_loop(&mut self, bind: usize, iter: IR, body: IR) -> Val {
        let iter = self.eval(iter);

        let ILitType::Array(iter) = iter else {
            unreachable!("arrays are the only iterable thing");
        };

        let mut last = ILitType::Unit;

        for rust_idx in iter.into_iter() {
            let inner_body = body.clone();
            self.push_frame();

            self.set_var(bind, rust_idx.clone());
            last = self.eval(inner_body);

            self.pop_frame();
        }

        last
    }

    fn eval_grouping<I>(&mut self, inner: I, redirect: Option<(Box<IR>, usize)>) -> Val
    where
        I: IntoIterator<Item = IR>,
    {
        self.push_frame();

        if let Some((redir_ir, bind)) = redirect {
            let val = self.eval(*redir_ir);
            self.set_var(bind, val);
        }

        let mut last_val = ILitType::Unit;

        for node in inner {
            last_val = self.eval(node);
        }

        self.pop_frame();

        last_val
    }
}
