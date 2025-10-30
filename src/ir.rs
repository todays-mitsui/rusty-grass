use crate::ast;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Instr {
    App {
        func_idx: NonZeroUsize,
        arg_idx: NonZeroUsize,
    },
    Abs {
        arity: NonZeroUsize,
        body: Code,
    },
}

pub type Code = VecDeque<Instr>;

#[derive(Debug, Clone)]
pub enum Value {
    Char(u8),
    Closure { code: Code, env: Rc<Env> },
    Prim(Prim),
}

#[derive(Debug, Clone)]
pub enum Prim {
    In,
    Succ,
    Out,
}

#[derive(Debug, Clone)]
pub enum Env {
    Empty,
    Node(Value, Rc<Env>),
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub code: Code,
    pub env: Rc<Env>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub code: Code,
    pub env: Env,
    pub dump: Vec<Frame>,
}

// ========================================================================== //

impl From<&ast::Prog> for Code {
    fn from(prog: &ast::Prog) -> Self {
        prog.items.iter().map(Instr::from).collect()
    }
}

impl From<&ast::Top> for Instr {
    fn from(top: &ast::Top) -> Self {
        match top {
            ast::Top::Abs(abs) => Instr::from(abs),
            ast::Top::App(app) => Instr::from(app),
        }
    }
}

impl From<&ast::Abs> for Instr {
    fn from(abs: &ast::Abs) -> Self {
        Instr::Abs {
            arity: NonZeroUsize::new(abs.arity).unwrap(),
            body: abs.body.iter().map(Instr::from).collect(),
        }
    }
}

impl From<&ast::App> for Instr {
    fn from(app: &ast::App) -> Self {
        Instr::App {
            func_idx: NonZeroUsize::new(app.func_idx).unwrap(),
            arg_idx: NonZeroUsize::new(app.arg_idx).unwrap(),
        }
    }
}

impl Env {
    pub fn nil() -> Rc<Self> {
        Rc::new(Env::Empty)
    }

    pub fn push(self: &Rc<Self>, v: Value) -> Rc<Env> {
        Rc::new(Env::Node(v, self.clone()))
    }

    pub fn get(self: &Rc<Self>, idx: NonZeroUsize) -> Option<&Value> {
        let mut env = self.as_ref();
        for _ in 1..idx.get() {
            match env {
                Env::Empty => return None,
                Env::Node(_, next) => env = next.as_ref(),
            }
        }
        match env {
            Env::Empty => None,
            Env::Node(v, _) => Some(v),
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Env::Empty
    }
}
