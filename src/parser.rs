use crate::ast;
use combine::stream::Stream;
use combine::stream::position::{SourcePosition, Stream as PositionStream};
use combine::{
    Parser, error::StringStreamError, many, many1, none_of, one_of, optional, position, skip_many,
};
use std::iter;

pub fn parse_prog(input: &str) -> Result<ast::Prog, StringStreamError> {
    let stream = PositionStream::new(input);
    match prog().parse(stream) {
        Ok((prog, _remaining_input)) => Ok(prog),
        Err(err) => Err(err),
    }
}

// ========================================================================== //

fn white_space<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
{
    skip_many(none_of("wWvｗＷｖ".chars()))
}

fn head_white_space<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
{
    skip_many(none_of("wｗ".chars()))
}

fn char_w<Input>() -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    one_of("wｗ".chars()).skip(white_space()).map(|_| 'w')
}

#[allow(non_snake_case)]
fn char_W<Input>() -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    one_of("WＷ".chars()).skip(white_space()).map(|_| 'W')
}

fn char_v<Input>() -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    one_of("vｖ".chars()).skip(white_space()).map(|_| 'v')
}

fn app<'a>() -> impl Parser<PositionStream<&'a str, SourcePosition>, Output = ast::App> {
    let func_idx = many1::<Vec<_>, _, _>(char_W()).map(|ws| ws.len());
    let arg_idx = many1::<Vec<_>, _, _>(char_w()).map(|ws| ws.len());

    (position(), func_idx, arg_idx, position()).map(|(start_pos, func_idx, arg_idx, end_pos)| {
        ast::App {
            func_idx,
            arg_idx,
            range: ast::SourceRange {
                start: start_pos,
                end: end_pos,
            },
        }
    })
}

fn abs<'a>() -> impl Parser<PositionStream<&'a str, SourcePosition>, Output = ast::Abs> {
    let arity = many1::<Vec<_>, _, _>(char_w()).map(|ws| ws.len());
    let body = many(app());

    (position(), arity, body, position()).map(|(start_pos, arity, body, end_pos)| ast::Abs {
        arity,
        body,
        range: ast::SourceRange {
            start: start_pos,
            end: end_pos,
        },
    })
}

fn prog<'a>() -> impl Parser<PositionStream<&'a str, SourcePosition>, Output = ast::Prog> {
    let head = abs().map(ast::Top::Abs);

    let top = abs().map(ast::Top::Abs).or(app().map(ast::Top::App));
    let tail = many::<Vec<_>, _, _>(optional(char_v()).with(top));

    (head_white_space(), head, tail).map(|(_, head, tail)| {
        // 先頭が head, それに tail が続く Vec<ast::Top> を作る
        let items = iter::once(head).chain(tail.iter().cloned()).collect();
        ast::Prog { items }
    })
}

// ========================================================================== //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app() {
        let input = "WWWwwww";
        let result = app().parse(PositionStream::new(input));
        assert!(result.is_ok());
        let (app, _) = result.unwrap();
        assert_eq!(
            app,
            ast::App {
                func_idx: 3,
                arg_idx: 4,
                range: ast::SourceRange {
                    start: SourcePosition { line: 1, column: 1 },
                    end: SourcePosition { line: 1, column: 8 },
                },
            }
        );
    }

    #[test]
    fn test_abs() {
        let input = "wwwwwWWwwwwWwww";
        let result = abs().parse(PositionStream::new(input));
        assert!(result.is_ok());
        let (abs, _) = result.unwrap();
        assert_eq!(
            abs,
            ast::Abs {
                arity: 5,
                body: vec![
                    ast::App {
                        func_idx: 2,
                        arg_idx: 4,
                        range: ast::SourceRange {
                            start: SourcePosition { line: 1, column: 6 },
                            end: SourcePosition {
                                line: 1,
                                column: 12
                            },
                        },
                    },
                    ast::App {
                        func_idx: 1,
                        arg_idx: 3,
                        range: ast::SourceRange {
                            start: SourcePosition {
                                line: 1,
                                column: 12
                            },
                            end: SourcePosition {
                                line: 1,
                                column: 16
                            },
                        },
                    },
                ],
                range: ast::SourceRange {
                    start: SourcePosition { line: 1, column: 1 },
                    end: SourcePosition {
                        line: 1,
                        column: 16
                    },
                },
            }
        );
    }

    #[test]
    fn test_prog() {
        let input = "wWWwwwvWWWWwwwwwWWwvwwWwwWWWwwwwwWWWWWwwwwww";
        let result = prog().parse(PositionStream::new(input));
        assert!(result.is_ok());
        let (prog, _) = result.unwrap();
        assert_eq!(prog.items.len(), 4);
        assert_eq!(
            prog.items[0],
            ast::Top::Abs(ast::Abs {
                arity: 1,
                body: vec![ast::App {
                    func_idx: 2,
                    arg_idx: 3,
                    range: ast::SourceRange {
                        start: SourcePosition { line: 1, column: 2 },
                        end: SourcePosition { line: 1, column: 7 },
                    },
                },],
                range: ast::SourceRange {
                    start: SourcePosition { line: 1, column: 1 },
                    end: SourcePosition { line: 1, column: 7 },
                },
            })
        );
        assert_eq!(
            prog.items[1],
            ast::Top::App(ast::App {
                func_idx: 4,
                arg_idx: 5,
                range: ast::SourceRange {
                    start: SourcePosition { line: 1, column: 8 },
                    end: SourcePosition {
                        line: 1,
                        column: 17
                    }
                }
            }),
        );
        assert_eq!(
            prog.items[2],
            ast::Top::App(ast::App {
                func_idx: 2,
                arg_idx: 1,
                range: ast::SourceRange {
                    start: SourcePosition {
                        line: 1,
                        column: 17
                    },
                    end: SourcePosition {
                        line: 1,
                        column: 20
                    }
                }
            }),
        );
        assert_eq!(
            prog.items[3],
            ast::Top::Abs(ast::Abs {
                arity: 2,
                body: vec![
                    ast::App {
                        func_idx: 1,
                        arg_idx: 2,
                        range: ast::SourceRange {
                            start: SourcePosition {
                                line: 1,
                                column: 23
                            },
                            end: SourcePosition {
                                line: 1,
                                column: 26
                            }
                        }
                    },
                    ast::App {
                        func_idx: 3,
                        arg_idx: 5,
                        range: ast::SourceRange {
                            start: SourcePosition {
                                line: 1,
                                column: 26
                            },
                            end: SourcePosition {
                                line: 1,
                                column: 34
                            }
                        }
                    },
                    ast::App {
                        func_idx: 5,
                        arg_idx: 6,
                        range: ast::SourceRange {
                            start: SourcePosition {
                                line: 1,
                                column: 34
                            },
                            end: SourcePosition {
                                line: 1,
                                column: 45
                            }
                        }
                    }
                ],
                range: ast::SourceRange {
                    start: SourcePosition {
                        line: 1,
                        column: 21
                    },
                    end: SourcePosition {
                        line: 1,
                        column: 45
                    }
                }
            })
        );
    }
}
