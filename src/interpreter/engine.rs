use std::collections::HashMap;

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

#[derive(Debug)]
pub struct Engine<'a> {
    ir: &'a [IR],
    func_table: &'a FuncTable<'a>,
    // Even though the IR generator ensures that all variables have unique identifiers, we still
    // need to have stack frames for recursion in function calls.
    frames: Vec<StackFrame>,
    funcs: Functions<'a>,
}

#[derive(Debug, Clone, Default)]
pub struct StackFrame {
    vars: HashMap<String, ILitType>,
}

impl<'a> Engine<'a> {
    pub fn new(ir: &'a [IR], func_table: &'a FuncTable<'a>) -> Self {
        Self {
            ir,
            func_table,
            frames: vec![StackFrame::default()],
            funcs: Functions::stdlib(),
        }
    }

    fn get_var(&self, name: &str) -> Option<&ILitType> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.vars.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn set_var<T: ToString>(&mut self, name: T, val: ILitType) {
        self.frames
            .last_mut()
            .unwrap()
            .vars
            .insert(name.to_string(), val);
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

    fn eval(&mut self, node: &'a IR) -> Option<ILitType> {
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

                self.funcs.insert(name, func);

                None
            }
            IR::Grouping {
                inner,
                expr_end,
                redirect,
            } => {
                self.push_frame();

                if let Some((redir_ir, bind)) = redirect {
                    let val = self
                        .eval(redir_ir)
                        .expect("redirect did not produce value!!!");
                    self.set_var(bind, val);
                }

                for node in inner {
                    self.eval(node);
                }

                let inner_val = match expr_end {
                    Some(expr) => self.eval(expr),
                    None => Some(ILitType::Unit),
                };

                self.pop_frame();

                inner_val
            }
            IR::For { bind, iter, body } => self.eval_for_loop(bind, iter, body),
            IR::Let { name, ty: _, value } => {
                debug_assert_eq!(value.len(), 1);
                let val = self.eval(&value[0]).expect("did not produce value!!!");
                self.set_var(name, val.clone());
                Some(val)
            }
            IR::Match { expr, arms } => {
                let expr = self
                    .eval(&expr[0])
                    .expect("match expr did not produce value");

                let ILitType::Result(result) = expr else {
                    panic!("type checked");
                };

                let mut last = None;

                for arm in arms {
                    let (bind_name, is_ok, body) = {
                        let IRMatchArm { bind, is_ok, body } = arm;
                        (bind, is_ok, body)
                    };

                    let active = match (&result, is_ok) {
                        (IResultBranch::Ok(v), true) => Some(v.clone()),
                        (IResultBranch::Err(v), false) => Some(v.clone()),
                        _ => None,
                    };

                    if let Some(val) = active {
                        self.push_frame();
                        self.set_var(bind_name, *val);

                        for node in body {
                            last = self.eval(node);
                        }

                        self.pop_frame();
                        return last;
                    }
                }

                panic!("match didn't find an arm");
            }
            IR::FuncCall { name, args, unwrap } => {
                let func = self
                    .funcs
                    .resolve(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("function `{name}` not found!"));

                let mut evaled_args = Vec::with_capacity(args.len());

                for arg in args {
                    let val = self.eval(arg).expect("arg produced no value");
                    evaled_args.push(val);
                }

                let ret = match func {
                    RuntimeFunc::Internal(f) => f(self, &evaled_args),
                    RuntimeFunc::User(func) => {
                        self.push_frame();

                        for (i, (arg_name, _)) in func.args.iter().enumerate() {
                            let val = evaled_args.get(i).cloned().unwrap_or(ILitType::Unit);

                            self.set_var(arg_name, val);
                        }

                        let mut last = None;

                        for node in func.body {
                            last = self.eval(node);
                        }

                        self.pop_frame();

                        last
                    }
                };

                if *unwrap {
                    match ret.expect("return failed") {
                        ILitType::Result(iresult_branch) => match iresult_branch {
                            IResultBranch::Ok(ilit_type) => Some(*ilit_type),
                            IResultBranch::Err(_) => panic!("found err branch"),
                        },
                        err => panic!("expected `result`, but got `{:?}`", err),
                    }
                } else {
                    ret
                }
            }
            IR::Integer(i) => Some(ILitType::Integer(*i)),
            IR::String(s) => Some(ILitType::String(s.to_string())),
            IR::Ident(ident) => Some(self.get_var(ident).cloned().expect("could not find ident")),
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

    fn eval_for_loop(&mut self, bind: &str, iter: &'a [IR], body: &'a [IR]) -> Option<ILitType> {
        debug_assert_eq!(iter.len(), 1);

        let iter = self.eval(&iter[0]).expect("iter did not produce value");

        let ILitType::Array(iter) = iter else {
            unreachable!("arrays are the only iterable thing");
        };

        let mut last = None;

        for rust_idx in iter {
            self.push_frame();

            self.set_var(bind, rust_idx);
            for ir in body {
                last = self.eval(ir);
            }

            self.pop_frame();
        }

        last
    }
}
