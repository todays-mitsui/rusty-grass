use crate::ast::Prog;
use crate::ir::{self, Prim, Value};
use std::collections::VecDeque;
use std::io::Read;
use std::num::NonZeroUsize;
use std::rc::Rc;
use thiserror::Error;

pub struct VM {
    code: ir::Code,
    env: Rc<ir::Env>,
    dump: Vec<ir::Frame>,
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
        let code = ir::Code::from(prog);

        let env = ir::Env::nil()
            .push(Value::Prim(Prim::Out))
            .push(Value::Prim(Prim::Succ))
            .push(Value::Char(b'w'))
            .push(Value::Prim(Prim::In));

        let dump0 = vec![ir::Frame {
            code: VecDeque::new(),
            env: ir::Env::nil(),
        }];

        Self {
            code,
            env,
            dump: dump0,
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.code.pop_front() {
                Some(instr) => match instr {
                    ir::Instr::App { func_idx, arg_idx } => {
                        let ff = self
                            .env
                            .get(func_idx)
                            .ok_or_else(|| RuntimeError::IndexOutOfBounds(func_idx))
                            .cloned()?;
                        let fa = self
                            .env
                            .get(arg_idx)
                            .ok_or_else(|| RuntimeError::IndexOutOfBounds(arg_idx))
                            .cloned()?;
                        self.call(ff, fa)?;
                    }
                    ir::Instr::Abs { arity, body } => {
                        if arity.get() == 1 {
                            self.env = self.env.push(Value::Closure {
                                code: body,
                                env: self.env.clone(),
                            });
                        } else {
                            let decrement = ir::Instr::Abs {
                                arity: NonZeroUsize::new(arity.get() - 1).unwrap(),
                                body,
                            };
                            self.env = self.env.push(Value::Closure {
                                code: VecDeque::from(vec![decrement]),
                                env: self.env.clone(),
                            });
                        }
                    }
                },
                None => {
                    if let Some(frame) = self.dump.pop() {
                        let return_value = self
                            .env
                            .get(NonZeroUsize::new(1).unwrap())
                            .cloned()
                            .ok_or_else(|| RuntimeError::IllegalState)?;
                        self.code = frame.code;
                        self.env = frame.env.push(return_value);
                        continue;
                    }

                    let result_value = self
                        .env
                        .get(NonZeroUsize::new(1).unwrap())
                        .cloned()
                        .ok_or_else(|| RuntimeError::IllegalState)?;
                    let self_value = result_value.clone();
                    match result_value {
                        Value::Closure { code, env } => {
                            self.code = code;
                            self.env = env.push(self_value);
                            continue;
                        }
                        _ => return Ok(()),
                    }
                }
            }
        }
    }

    fn call(&mut self, func: Value, arg: Value) -> Result<(), RuntimeError> {
        match func {
            Value::Char(expected) => {
                let return_value = match arg {
                    Value::Char(actual) if expected == actual => church_true(),
                    _ => church_false(),
                };
                self.env = self.env.push(return_value);
            }
            Value::Closure { code, env } => {
                let frame = ir::Frame {
                    code: std::mem::take(&mut self.code),
                    env: std::mem::take(&mut self.env),
                };
                self.dump.push(frame);

                self.code = code;
                self.env = env.push(arg);
            }
            Value::Prim(prim) => {
                let result_value = self.call_prim(prim, arg)?;
                self.env = self.env.push(result_value);
            }
        }
        Ok(())
    }

    fn call_prim(&mut self, prim: Prim, arg: Value) -> Result<Value, RuntimeError> {
        match prim {
            Prim::In => {
                let mut buf = [0u8; 1];
                match std::io::stdin().read(&mut buf) {
                    Ok(1) => Ok(Value::Char(buf[0])),
                    _ => Ok(arg),
                }
            }
            Prim::Succ => {
                if let Value::Char(char) = arg {
                    Ok(Value::Char(char.wrapping_add(1)))
                } else {
                    Err(RuntimeError::NotAChar(arg))
                }
            }
            Prim::Out => {
                if let Value::Char(c) = arg {
                    print!("{}", c as char);
                    Ok(arg)
                } else {
                    Err(RuntimeError::NotAChar(arg))
                }
            }
        }
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
