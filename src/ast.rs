use combine::stream::position::SourcePosition;

#[derive(Debug, PartialEq, Clone)]
pub struct Prog {
    pub items: Vec<Top>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Top {
    Abs(Abs),
    App(App),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Abs {
    pub arity: usize,
    pub body: Vec<App>,
    pub range: SourceRange,
}

#[derive(Debug, PartialEq, Clone)]
pub struct App {
    pub func_idx: usize,
    pub arg_idx: usize,
    pub range: SourceRange,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}
