use crate::ir::*;
use std::fmt::Debug;
use std::rc::Rc;

pub struct PP<'a, T>(pub &'a T);

impl<'a> Debug for PP<'a, State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("code", &PP(&self.0.code))
            .field("env", &PP(&self.0.env))
            .field("dump", &PP(&self.0.dump))
            .finish()
    }
}

impl<'a> Debug for PP<'a, Code> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        let mut iter = self.0.iter();
        // 高々3要素まで表示し、残りがあれば省略表示する
        for _ in 0..3 {
            if let Some(instr) = iter.next() {
                list.entry(&PP(instr));
            } else {
                break;
            }
        }
        if iter.next().is_some() {
            list.entry(&Ellipsis);
        }
        list.finish()
    }
}

impl<'a> Debug for PP<'a, Instr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Instr::App { func_idx, arg_idx } => f
                .debug_tuple("_App_")
                .field(func_idx)
                .field(arg_idx)
                .finish(),
            Instr::Abs { arity, body } => f
                .debug_tuple("_Abs_")
                .field(arity)
                .field(&PP(body))
                .finish(),
        }
    }
}

impl<'a> Debug for PP<'a, Rc<Env>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut env = self.0.as_ref();
        let mut list = f.debug_list();
        let mut index = 1;
        while let Env::Node(v, next) = env {
            // 高々3要素まで表示し、残りがあれば省略表示する
            if index > 3 {
                list.entry(&Ellipsis);
                break;
            }
            list.entry(&PP(&(index, v)));
            env = next.as_ref();
            index += 1;
        }
        list.finish()
    }
}

impl<'a> Debug for PP<'a, Vec<Frame>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|frame| PP(frame)))
            .finish()
    }
}

impl<'a> Debug for PP<'a, Frame> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("code", &PP(&self.0.code))
            .field("env", &PP(&self.0.env))
            .finish()
    }
}

impl<'a> Debug for PP<'a, (usize, &Value)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0.0, f)?;
        f.write_str(": ")?;
        Debug::fmt(&PP(self.0.1), f)
    }
}

impl<'a> Debug for PP<'a, Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Char(c) => Debug::fmt(&(*c as char), f),
            Value::Closure { code, env } => f
                .debug_struct("Closure")
                .field("code", &PP(code))
                .field("env", &PP(env))
                .finish(),
            Value::Prim(prim) => Debug::fmt(&PP(prim), f),
        }
    }
}

impl<'a> Debug for PP<'a, Prim> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Prim::In => write!(f, "In"),
            Prim::Succ => write!(f, "Succ"),
            Prim::Out => write!(f, "Out"),
        }
    }
}

struct Ellipsis;

impl Debug for Ellipsis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "...")
    }
}
