use crate::ast::Prog;
use crate::ir::{self, Prim, Value};
use crate::pp::PP;
use std::collections::VecDeque;
use std::io::Read;
use std::num::NonZeroUsize;
use thiserror::Error;
use tracing::debug;

pub struct VM {
    state: ir::State,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("out of bounds access at index {0}")]
    IndexOutOfBounds(NonZeroUsize),
    #[error("illegal state encountered")]
    IllegalState,
    #[error("expected a character value, found {0:?}")]
    NotAChar(Value),
}

impl VM {
    pub fn new(prog: &Prog) -> Self {
        let code0 = ir::Code::from(prog);

        let env0 = ir::Env::nil()
            .push(Value::Prim(Prim::In))
            .push(Value::Char(b'w'))
            .push(Value::Prim(Prim::Succ))
            .push(Value::Prim(Prim::Out));

        let dump0 = vec![ir::Frame {
            code: VecDeque::new(),
            env: ir::Env::nil(),
        }];

        let state = ir::State {
            code: code0,
            env: env0,
            dump: dump0,
        };

        debug!("init: {:?}", PP(&state));

        Self { state }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            debug!("loop: {:?}", PP(&self.state));

            match self.state.code.pop_front() {
                Some(instr) => match instr {
                    ir::Instr::App { func_idx, arg_idx } => {
                        let ff = self
                            .state
                            .env
                            .get(func_idx)
                            .ok_or_else(|| RuntimeError::IndexOutOfBounds(func_idx))
                            .cloned()?;
                        let fa = self
                            .state
                            .env
                            .get(arg_idx)
                            .ok_or_else(|| RuntimeError::IndexOutOfBounds(arg_idx))
                            .cloned()?;
                        self.call(ff, fa)?;
                    }
                    ir::Instr::Abs { arity, body } => {
                        if arity.get() == 1 {
                            self.state.env = self.state.env.push(Value::Closure {
                                code: body,
                                env: self.state.env.clone(),
                            });
                        } else {
                            let decrement = ir::Instr::Abs {
                                arity: NonZeroUsize::new(arity.get() - 1).unwrap(),
                                body,
                            };
                            self.state.env = self.state.env.push(Value::Closure {
                                code: VecDeque::from(vec![decrement]),
                                env: self.state.env.clone(),
                            });
                        }
                    }
                },
                None => {
                    if let Some(frame) = self.state.dump.pop() {
                        let return_value = self
                            .state
                            .env
                            .get(NonZeroUsize::new(1).unwrap())
                            .cloned()
                            .ok_or_else(|| RuntimeError::IllegalState)?;
                        self.state.code = frame.code;
                        self.state.env = frame.env.push(return_value);
                        continue;
                    }

                    let result_value = self
                        .state
                        .env
                        .get(NonZeroUsize::new(1).unwrap())
                        .cloned()
                        .ok_or_else(|| RuntimeError::IllegalState)?;
                    let self_value = result_value.clone();
                    match result_value {
                        Value::Closure { code, env } => {
                            self.state.code = code;
                            self.state.env = env.push(self_value);
                            continue;
                        }
                        _ => return Ok(()),
                    }
                }
            }
        }
    }

    fn call(&mut self, func: Value, arg: Value) -> Result<(), RuntimeError> {
        debug!("call: func: {:?}, arg: {:?}", PP(&func), PP(&arg));
        match func {
            Value::Char(expected) => {
                let return_value = match arg {
                    Value::Char(actual) if expected == actual => church_true(),
                    _ => church_false(),
                };
                self.state.env = self.state.env.push(return_value);
            }
            Value::Closure { code, env } => {
                let frame = ir::Frame {
                    code: std::mem::take(&mut self.state.code),
                    env: std::mem::take(&mut self.state.env),
                };
                self.state.dump.push(frame);

                self.state.code = code;
                self.state.env = env.push(arg);
            }
            Value::Prim(prim) => {
                let result_value = match prim {
                    Prim::In => {
                        let mut buf = [0u8; 1];
                        match std::io::stdin().read(&mut buf) {
                            Ok(1) => {
                                debug!("io: stdin: byte={} {:?}", buf[0], buf[0] as char);
                                Value::Char(buf[0])
                            }
                            _ => arg,
                        }
                    }
                    Prim::Succ => {
                        if let Value::Char(char) = arg {
                            Value::Char(char.wrapping_add(1))
                        } else {
                            return Err(RuntimeError::NotAChar(arg));
                        }
                    }
                    Prim::Out => {
                        if let Value::Char(c) = arg {
                            print!("{}", c as char);
                            debug!("io: stdout: byte={} {:?}", c, c as char);
                            arg
                        } else {
                            return Err(RuntimeError::NotAChar(arg));
                        }
                    }
                };
                self.state.env = self.state.env.push(result_value);
            }
        }
        Ok(())
    }
}

// ========================================================================== //

fn identity() -> ir::Value {
    ir::Value::Closure {
        code: VecDeque::new(),
        env: ir::Env::nil(),
    }
}

fn church_false() -> ir::Value {
    let code = VecDeque::from(vec![ir::Instr::Abs {
        arity: NonZeroUsize::new(1).unwrap(),
        body: VecDeque::new(),
    }]);
    ir::Value::Closure {
        code,
        env: ir::Env::nil(),
    }
}

fn church_true() -> ir::Value {
    let code = VecDeque::from(vec![ir::Instr::Abs {
        arity: NonZeroUsize::new(1).unwrap(),
        body: VecDeque::from(vec![ir::Instr::App {
            func_idx: NonZeroUsize::new(3).unwrap(),
            arg_idx: NonZeroUsize::new(2).unwrap(),
        }]),
    }]);
    ir::Value::Closure {
        code,
        env: ir::Env::nil().push(identity()),
    }
}
