use crate::evaluate::{compare, evaluate};
use crate::{Values, main_loop, tokenizer};

enum State {
    Normal,
    Singlequote,
    Doublequote,
    Backslash(Box<State>),
    CurlyBracket(Box<State>),
    SquareBracket(Box<State>),
    Bracket(Box<State>),
}

pub fn parse_arg(s: &str, values: &mut Values) -> Result<Vec<String>, &'static str> {
    use State::*;
    let mut s_ = String::from(s);
    let mut result = Vec::new();
    let mut temp = String::new();
    let mut slice = String::new();
    let mut state: State = Normal;
    s_.push('\n');
    loop {
        for c in s_.chars() {
            match state {
                Normal | Doublequote => match c {
                    '\\' => {
                        state = Backslash(Box::from(state));
                        continue;
                    }
                    '{' => {
                        state = CurlyBracket(Box::from(state));
                        continue;
                    }
                    '[' => {
                        state = SquareBracket(Box::from(state));
                        continue;
                    }
                    '(' => {
                        state = Bracket(Box::from(state));
                        continue;
                    }
                    _ => {}
                },
                _ => {}
            }
            match state {
                Normal => {
                    match c {
                        ' ' => {
                            result.push(slice.clone());
                        }
                        '\'' => state = Singlequote,
                        '\"' => state = Doublequote,
                        _ => slice.push(c),
                    };
                }
                Singlequote => match c {
                    '\'' => state = Normal,
                    _ => slice.push(c),
                },
                Doublequote => match c {
                    '\"' => state = Normal,
                    _ => slice.push(c),
                },
                Backslash(x) => {
                    if c == 'n' {
                        slice.push('\n');
                    } else {
                        slice.push('\\');
                        slice.push(c);
                    }
                    state = *x;
                }
                CurlyBracket(x) => match c {
                    '}' => {
                        if matches!(*x, CurlyBracket(_)) {
                            temp = evaluate(&temp, &values.vars);
                        } else {
                            slice += &evaluate(&temp, &values.vars);
                        }
                        state = *x;
                    }
                    '{' => {
                        state = CurlyBracket(Box::from(*x));
                        state = CurlyBracket(Box::from(state));
                    }
                    _ => {
                        temp.push(c);
                        state = CurlyBracket(Box::from(*x));
                    }
                },
                SquareBracket(x) => match c {
                    ']' => {
                        slice.push(compare(&temp, &values.vars));
                        state = *x;
                    }
                    _ => {
                        temp.push(c);
                        state = SquareBracket(Box::from(*x));
                    }
                },
                Bracket(x) => match c {
                    ')' => {
                        let t = tokenizer(temp.trim());
                        let result = main_loop(values, &t);
                        for r in result {
                            match r {
                                Ok(x) => slice += &x,
                                Err(x) => return Err(Box::leak(x.into_boxed_str())),
                            }
                        }
                        state = *x;
                    }
                    _ => {
                        temp.push(c);
                        state = Bracket(Box::from(*x));
                    }
                },
            }
        }
        match state {
            Normal => break,
            Doublequote | Singlequote => {
                print!("> ");
                s_ = crate::input();
            }
            CurlyBracket(_) => return Err("curly brace opened but never closed: expected }"),
            SquareBracket(_) => return Err("square bracket opened but never closed: expected ]"),
            Bracket(_) => return Err("square bracket opened but never closed: expected )"),
            _ => {}
        }
    }
    slice.pop();
    result.push(slice.clone());
    return Ok(result);
}
