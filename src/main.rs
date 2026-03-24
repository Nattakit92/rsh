use std::collections::HashMap;
use std::env;
use std::io;
use std::path::PathBuf;

mod commands;
pub mod parsing;

pub enum VarTypes {
    ///int type
    I(i32),
    ///string type
    S(String),
    /// none type
    N,
}

pub struct Values {
    dir: PathBuf,
    args: Option<Vec<String>>,
    vars: HashMap<String, VarTypes>,
}

pub fn normalise_dir(path: &PathBuf) -> PathBuf {
    let mut dir: PathBuf = PathBuf::new();
    for d in path {
        if d == ".." {
            dir.pop();
            continue;
        }
        if d == "." {
            continue;
        }
        dir.push(d);
    }
    return dir;
}

fn input() -> String {
    io::Write::flush(&mut io::stdout()).expect("flush failed!");
    let mut s = String::new();
    match io::stdin().read_line(&mut s) {
        Ok(_) => (),
        Err(err) => eprintln!("{}", err),
    };
    return s;
}

fn tokenizer(s: &str) -> Vec<&str> {
    match s.split_once(' ') {
        Some((a, b)) => return vec![a, b],
        None => return vec![s],
    }
}

fn main() {
    let mut values: Values = Values {
        dir: env::current_dir().unwrap(),
        args: None,
        vars: HashMap::new(),
    };
    loop {
        print!("{} $ ", values.dir.to_string_lossy());
        let s = input();
        if s == "\n" {
            continue;
        }
        let t: Vec<&str> = tokenizer(s.trim());
        if t.is_empty() {
            continue;
        }
        let command = commands::search(&t);
        if command.is_none() {
            eprintln!("Unknown command: {}", t[0]);
            continue;
        }
        if t.len() > 1 {
            match parsing::parse_arg(t[1], &values.vars) {
                Ok(x) => {
                    values.args = Some(x);
                    ()
                }
                Err(err) => {
                    eprintln!("{}: {}", t[0], err);
                    continue;
                }
            }
        } else {
            values.args = None;
        }
        let result = command.unwrap().run(&mut values);
        for r in result {
            match r {
                Ok(x) => print!("{}", x),
                Err(x) => eprint!("{}", x),
            }
        }
        values.args = None;
    }
}
