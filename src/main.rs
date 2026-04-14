use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, io};

mod commands;
mod parsing;
mod config;

pub mod evaluate;
pub mod input;

#[derive(Clone)]
pub enum VarTypes {
    ///int type
    I(i32),
    ///string type
    S(String),
    /// none type
    N,
}

impl VarTypes {
    pub fn get_i(&self) -> i32 {
        if let Self::I(i) = self {
            return *i;
        }
        return 0;
    }
    pub fn get_s(&self) -> String {
        match self {
            Self::I(x) => x.to_string(),
            Self::S(x) => x.clone(),
            Self::N => String::new(),
        }
    }
    pub fn get_type(&self) -> char {
        match self {
            Self::I(_) => 'I',
            Self::S(_) => 'S',
            Self::N => 'N',
        }
    }
}

#[derive(Clone)]
pub struct Values {
    dir: PathBuf,
    args: Option<Vec<String>>,
    vars: HashMap<String, VarTypes>,
    pipe: Option<String>,
    history: Vec<String>,
    stdout: bool,
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

fn main_loop(values: &mut Values, s: &str) -> (Vec<Result<String, String>>, Option<String>) {
    if s.is_empty() {
        return (vec![], None);
    }
    let args = parsing::parse_arg(s, values);
    let c: String;
    match args {
        Ok(mut x) => {
            c = x.remove(0);
            if x.is_empty() {
                values.args = None;
            } else {
                values.args = Some(x);
            }
        }
        Err(err) => {
            return (vec![Err(format!("{}", err))], None);
        }
    }

    let command = commands::search(&c);
    match command {
        Some(x) => (x.run(values), Some(c)),
        None => (vec![Err(format!("Unknown command: {}", c))], None),
    }
}

fn main() {
    let mut values: Values = Values {
        dir: env::current_dir().unwrap(),
        args: None,
        vars: HashMap::new(),
        pipe: None,
        history: Vec::new(),
        stdout: true,
    };
    let mut color = "\x1b[35m";
    loop {
        io::Write::flush(&mut io::stdout()).expect("flush failed!");
        print!(
            "\x1b[34m{}\n{}> \x1b[39m",
            values.dir.to_string_lossy(),
            color
        );
        let s = input::input(values.history.clone());
        if s == "\n" {
            continue;
        }
        values.history.push(s.clone());
        let (result, command) = main_loop(&mut values, s.trim());

        for r in result {
            match r {
                Ok(x) => {
                    print!("{}", x);
                    color = "\x1b[35m";
                }
                Err(x) => {
                    match command {
                        None => eprint!("{}", x),
                        Some(_) => eprint!("{}: {}", command.clone().unwrap(), x),
                    }
                    color = "\x1b[31m";
                }
            }
        }
        values.args = None;
        env::set_current_dir(&values.dir).expect("Invalid location");
    }
}
