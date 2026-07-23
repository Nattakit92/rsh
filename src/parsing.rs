use crate::{Values, commands};
use crate::evaluate::{compare, evaluate};
use crate::{main_loop};

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

pub fn parse_commands(s: &str, queue: &mut Values) -> Result<(), String>{
    use State::*;
    let s_ = String::from(s) + "\0";
    let mut line = String::new();
    let mut state: State = Normal;
    for c in s_.chars(){
        match state {
            Normal => match c {
                '\n' => queue.com_q.push_back(line.clone()),
                '\"' => state = Doublequote,
                '\'' => state = Singlequote,
                x => line.push(x)
            }
            Doublequote => match c {
                '\"' => state = Normal,
                _ => continue
            }
            Singlequote => match c {
                '\'' => state = Normal,
                _ => continue
            }
            _ => continue
        }
    }
    match state {
        Doublequote => Err(String::from("Doublequote opened but never closed: expected \"\n")),
        Singlequote => Err(String::from("Singlequote opened but never closed: expected \'\n")),
        _ => Ok(())
    }
}

pub fn parse_arg(values: &mut Values) -> Result<Vec<String>, String> {
    use State::*;
    let s = values.get_com() + "\0";
    let mut result = Vec::new();
    let mut temp = String::new();
    let mut slice = String::new();
    let mut state: State = Normal;
    for c in s.chars() {
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
                        temp = evaluate(&temp, values);
                    } else {
                        slice += &evaluate(&temp, values);
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
                    slice = slice + &compare(&temp, values);
                    state = *x;
                }
                _ => {
                    temp.push(c);
                    state = SquareBracket(Box::from(*x));
                }
            },
            Bracket(x) => match c {
                ')' => {
                    values.cur_com.stdout = false;
                    let result = main_loop(values, temp.trim());
                    for r in result {
                        match r {
                            Ok(x) => slice += &x,
                            Err(x) => return Err(x),
                        }
                    }
                    values.cur_com.args = None;
                    state = *x;
                }
                _ => {
                    temp.push(c);
                    state = Bracket(Box::from(*x));
                }
            },
            Pipe => {
                values.cur_com.stdout = false;
                let (_,stdin) = run(&slice, values, &mut result);
                for r in stdin {
                    match r {
                        Ok(x) => values.cur_com.pipe = Some(x),
                        Err(x) => return Err(x),
                    }
                }
                values.cur_com.args = None;
                state = Normal;
                slice = String::new();
                result = Vec::new();
                values.cur_com.stdout = true;
                temp = String::new();
            }
            And => match c {
                '&' => {
                    values.cur_com.stdout = false;
                    let (_,stdout) = run(&slice, values, &mut result);
                    for r in stdout {
                        match r {
                            Ok(x) => print!("{}", x),
                            Err(x) => return Err(x),
                        }
                    }
                    values.cur_com.args = None;
                    state = Normal;
                    slice = String::new();
                    result = Vec::new();
                    temp = String::new();
                }
                _ => {
                    let mut values_ = values.clone();
                    values.cur_com.stdout = false;
                    std::thread::spawn(move || {
                        let (com,stdout) = run(&slice, &mut values_, &mut result);
                        for r in stdout {
                            match r {
                                Ok(x) => println!("{}", x),
                                Err(x) => eprintln!("{}: {}", com, x),
                            }
                        }
                    });
                    values.cur_com.args = None;
                    state = Normal;
                    slice = String::new();
                    result = Vec::new();
                    values.cur_com.stdout = true;
                    temp = String::new();
                }
            },
            OutRedirect => match c {
                '>' => {
                    let (_,stdout) = run(&slice, values, &mut result);
                    result = vec![String::from("write"), String::from("-a")];
                    for r in stdout {
                        match r {
                            Ok(x) => values.cur_com.pipe = Some(x),
                            Err(x) => return Err(format!("{}", x)),
                        }
                    }
                    values.cur_com.args = None;
                    state = Normal;
                    slice = String::new();
                    values.cur_com.stdout = true;
                    temp = String::new();
                }
                _ => {
                    let (_,stdout) = run(&slice, values, &mut result);
                    result = vec![String::from("write")];
                    for r in stdout {
                        match r {
                            Ok(x) => values.cur_com.pipe = Some(x),
                            Err(x) => return Err(format!("{}", x)),
                        }
                    }
                    values.cur_com.args = None;
                    state = Normal;
                    slice = String::new();
                    values.cur_com.stdout = true;
                    temp = String::new();
                }
            }
        }
    }
    match state {
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
    slice.pop();
    result.push(slice.clone());
    return Ok(result);
}

fn run(slice: &str, values: &mut Values, result: &mut Vec<String>) -> (String, Vec<Result<String, String>>) {
    if result.len() == 0 {
        values.cur_com.command = String::from(slice);
    } else {
        values.cur_com.command = result.remove(0);
        if slice != String::new(){
            result.push(String::from(slice));
        }
    }
    if !result.is_empty() {
        values.cur_com.args = Some(result.clone());
    }

    if values.alias.contains_key(&values.cur_com.command){
        let args = values.cur_com.args.clone();
        let alias_val = values.alias[&values.cur_com.command].clone();
        values.com_q.push_front(alias_val);
        let mut new = parse_arg(values).unwrap();

        values.cur_com.command = new.remove(0);
        if !new.is_empty(){
            if let Some(s) = args {
                new.extend(s);
            }
            values.cur_com.args = Some(new);
        }
    }

    let command = commands::search(values.cur_com.command.clone());
    if command.is_none() {
        return (values.cur_com.command.clone(), vec![Err(format!("Unknown command: {}", values.cur_com.command))]);
    }
    (values.cur_com.command.clone(), command.unwrap().run(values))
}
