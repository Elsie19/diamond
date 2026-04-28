use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{
        stdlib::{Functions, RuntimeFunc, UserFunc},
        types::{ILitType, IResultBranch},
    },
    typing::{
        pass_one::FuncTable,
        strata::{IR, IRMatchArm},
    },
};

type Val = Option<ILitType>;

#[derive(Debug)]
pub struct Engine<'a> {
    ir: &'a [IR],
    func_table: &'a FuncTable<'a>,
    // Even though the IR generator ensures that all variables have unique identifiers, we still
    // need to have stack frames for recursion in function calls.
    frames: Vec<StackFrame>,
    funcs: Functions<'a>,
    argv: Rc<[ILitType]>,
}

#[derive(Debug, Clone, Default)]
pub struct StackFrame {
    vars: HashMap<Rc<str>, ILitType>,
}

impl<'a> Engine<'a> {
    pub fn new<I, T>(ir: &'a [IR], func_table: &'a FuncTable<'a>, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            ir,
            func_table,
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

    fn get_var(&self, name: &str) -> Option<&ILitType> {
        self.frames.iter().rev().find_map(|f| f.vars.get(name))
    }

    fn set_var<T: Into<Rc<str>>>(&mut self, name: T, val: ILitType) {
        self.frames
            .last_mut()
            .expect("popped top frame, ruh roh")
            .vars
            .insert(name.into(), val);
    }

    fn push_frame(&mut self) {
        self.frames.push(StackFrame::default());
    }

    fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub fn run(&mut self) {
        for node in self.ir {
            self.eval(node);
        }
    }

    fn eval(&mut self, node: &'a IR) -> Val {
        match node {
            IR::FuncLet {
                name,
                args,
                internal: _,
                ret,
                body,
            } => {
                let func = RuntimeFunc::User(UserFunc {
                    args: args.clone().into_boxed_slice(),
                    body,
                    ret: ret.clone(),
                });

                self.funcs.insert(Rc::clone(name), func);

                None
            }
            IR::Grouping { inner, redirect } => self.eval_grouping(
                inner,
                redirect
                    .as_ref()
                    .map(|(ir, bind)| (ir.as_ref(), bind.as_ref())),
            ),
            IR::For { bind, iter, body } => self.eval_for_loop(bind, iter, body),
            IR::Let { name, ty: _, value } => {
                let val = self.eval(value).expect("did not produce value!!!");
                self.set_var(Rc::clone(name), val.clone());
                Some(val)
            }
            IR::Match { expr, arms } => self.eval_match(expr, arms),
            IR::FuncCall { name, args, unwrap } => self.eval_funccall(name, args, *unwrap),
            IR::Integer(i) => Some(ILitType::Integer(*i)),
            IR::String(s) => Some(ILitType::String(Rc::clone(s))),
            IR::Ident(ident) => Some(self.get_var(ident).cloned().expect("could not find ident")),
            IR::Array(irs) => {
                let elems = irs
                    .iter()
                    .map(|x| self.eval(x))
                    .collect::<Option<Vec<_>>>()
                    .expect("array did not return a value");
                Some(ILitType::Array(elems.into()))
            }
            IR::Unit => Some(ILitType::Unit),
            IR::Result { ok, err } => todo!("result"),
            IR::Expr(ir) => Some(self.eval(ir)?),
            IR::Stmt(ir) => {
                self.eval(ir)?;
                Some(ILitType::Unit)
            }
        }
    }

    fn eval_funccall<I>(&mut self, name: &str, args: I, unwrap: bool) -> Val
    where
        I: IntoIterator<Item = &'a IR>,
    {
        let func = self.funcs.resolve(name).cloned().unwrap_or_else(|| {
            panic!("function `{name}` not found! Did you add the internal function yet?")
        });

        let evaled_args = args
            .into_iter()
            .map(|x| self.eval(x))
            .collect::<Option<Vec<_>>>()
            .expect("arg produced no value");

        let ret = match func {
            RuntimeFunc::Internal(f) => f(self, &evaled_args),
            RuntimeFunc::User(func) => {
                self.push_frame();

                for (i, (arg_name, _)) in func.args.iter().enumerate() {
                    let val = evaled_args.get(i).cloned().unwrap_or(ILitType::Unit);

                    self.set_var(Rc::clone(arg_name), val);
                }

                let last = self.eval(func.body);

                self.pop_frame();

                last
            }?,
        };

        if unwrap {
            match ret {
                ILitType::Result(iresult_branch) => match iresult_branch {
                    IResultBranch::Ok(ilit_type) => Some(*ilit_type),
                    IResultBranch::Err(ilit_type) => {
                        use crate::interpreter::functions::system::panic as internal_panic;
                        internal_panic(self, &[*ilit_type, ILitType::Array(Rc::new([]))]);
                        unreachable!("panicked above???");
                    }
                },
                err => panic!("expected `result`, but got `{:?}`", err),
            }
        } else {
            Some(ret)
        }
    }

    fn eval_match(&mut self, expr: &'a IR, arms: &'a [IRMatchArm]) -> Val {
        let expr = self.eval(expr).expect("match expr did not produce value");

        let ILitType::Result(result) = expr else {
            unreachable!("type checked");
        };

        for arm in arms {
            let IRMatchArm { bind, is_ok, body } = arm;

            let active = match (&result, is_ok) {
                (IResultBranch::Ok(v), true) => Some(v.clone()),
                (IResultBranch::Err(v), false) => Some(v.clone()),
                _ => None,
            };

            if let Some(val) = active {
                self.push_frame();
                self.set_var(Rc::clone(bind), *val);

                let last = self.eval(body);

                self.pop_frame();
                return last;
            }
        }

        unreachable!("match didn't find an arm");
    }

    fn eval_for_loop(&mut self, bind: &str, iter: &'a IR, body: &'a IR) -> Val {
        let iter = self.eval(iter).expect("iter did not produce value");

        let ILitType::Array(iter) = iter else {
            unreachable!("arrays are the only iterable thing");
        };

        let mut last = None;

        for rust_idx in &*iter {
            self.push_frame();

            self.set_var(bind, rust_idx.clone());
            last = self.eval(body);

            self.pop_frame();
        }

        last
    }

    fn eval_grouping(&mut self, inner: &'a [IR], redirect: Option<(&'a IR, &'a str)>) -> Val {
        self.push_frame();

        if let Some((redir_ir, bind)) = redirect {
            let val = self
                .eval(redir_ir)
                .expect("redirect did not produce value!!!");
            self.set_var(bind, val);
        }

        let mut last_val = Some(ILitType::Unit);

        for node in inner {
            last_val = self.eval(node);
        }

        self.pop_frame();

        last_val
    }
}
