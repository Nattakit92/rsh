use crate::commands;
use crate::evaluate::{compare, evaluate};
use crate::{Values, main_loop};

enum State {
    Normal,
    Singlequote,
    Doublequote,
    Backslash(Box<State>),
    CurlyBracket(Box<State>),
    SquareBracket(Box<State>),
    Bracket(Box<State>),
    Pipe,
    And,
    OutRedirect,
}

pub fn parse_arg(s: &str, values: &mut Values) -> Result<Vec<String>, String> {
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
                            if slice == String::new() {
                                continue;
                            }
                            result.push(slice.clone());
                            slice = String::new();
                        }
                        '\'' => state = Singlequote,
                        '\"' => state = Doublequote,
                        '|' => state = Pipe,
                        '&' => state = And,
                        '>' => state = OutRedirect,
                        '\n' => {
                            let stdout = run(&slice, values, &mut result);
                            for r in stdout {
                                match r {
                                    Ok(x) => print!("{}", x),
                                    Err(x) => return Err(x),
                                }
                            }
                            values.args = None;
                            state = Normal;
                            slice = String::new();
                            result = Vec::new();
                            continue;
                        }
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
                        let (result, _) = main_loop(values, temp.trim());
                        for r in result {
                            match r {
                                Ok(x) => slice += &x,
                                Err(x) => return Err(x),
                            }
                        }
                        state = *x;
                    }
                    _ => {
                        temp.push(c);
                        state = Bracket(Box::from(*x));
                    }
                },
                Pipe => {
                    let stdin = run(&slice, values, &mut result);
                    for r in stdin {
                        match r {
                            Ok(x) => values.pipe = Some(x),
                            Err(x) => return Err(x),
                        }
                    }
                    values.args = None;
                    state = Normal;
                    slice = String::new();
                    result = Vec::new();
                    values.stdout = true;
                }
                And => match c {
                    '&' => {
                        let stdout = run(&slice, values, &mut result);
                        for r in stdout {
                            match r {
                                Ok(x) => print!("{}", x),
                                Err(x) => return Err(x),
                            }
                        }
                        values.args = None;
                        state = Normal;
                        slice = String::new();
                        result = Vec::new();
                    }
                    _ => {
                        let mut values_ = values.clone();
                        values.stdout = false;
                        std::thread::spawn(move || {
                            let stdout = run(&slice, &mut values_, &mut result);
                            for r in stdout {
                                match r {
                                    Ok(x) => println!("{}", x),
                                    Err(x) => eprintln!("{}", x),
                                }
                            }
                        });
                        values.args = None;
                        state = Normal;
                        slice = String::new();
                        result = Vec::new();
                        values.stdout = true;
                    }
                },
                OutRedirect => match c {
                    '>' => {
                        let stdout = run(&slice, values, &mut result);
                        result = vec![String::from("write"), String::from("-a")];
                        for r in stdout {
                            match r {
                                Ok(x) => values.pipe = Some(x),
                                Err(x) => return Err(format!("{}", x)),
                            }
                        }
                        values.args = None;
                        state = Normal;
                        slice = String::new();
                        values.stdout = true;
                    }
                    _ => {
                        let stdout = run(&slice, values, &mut result);
                        result = vec![String::from("write")];
                        for r in stdout {
                            match r {
                                Ok(x) => values.pipe = Some(x),
                                Err(x) => return Err(format!("{}", x)),
                            }
                        }
                        values.args = None;
                        state = Normal;
                        slice = String::new();
                        values.stdout = true;
                    }
                },
            }
        }
        match state {
            Normal => break,
            Doublequote | Singlequote => {
                print!("> ");
                s_ = crate::input::input(values.history.clone());
            }
            CurlyBracket(_) => {
                return Err(format!(
                    "curly brace opened but never closed: expected }}\n"
                ));
            }
            SquareBracket(_) => {
                return Err(format!(
                    "square bracket opened but never closed: expected ]\n"
                ));
            }
            Bracket(_) => {
                return Err(format!(
                    "square bracket opened but never closed: expected )\n"
                ));
            }
            _ => {}
        }
    }
    slice.pop();
    result.push(slice.clone());
    return Ok(result);
}

fn run(slice: &str, values: &mut Values, result: &mut Vec<String>) -> Vec<Result<String, String>> {
    let t;
    let result_ = result.clone();
    if result.len() == 0 {
        t = slice;
    } else {
        t = &result_[0];
        result.remove(0);
        if slice != String::new(){
            result.push(String::from(slice));
        }
    }
    if !result.is_empty() {
        values.args = Some(result.clone());
    }
    let command = commands::search(&t, values);
    if command.is_none() {
        return vec![Err(format!("Unknown command: {}", t))];
    }
    command.unwrap().run(values)
}
