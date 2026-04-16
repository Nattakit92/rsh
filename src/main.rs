use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::{env, io};

use crate::config::{get_history, store_history};

mod parsing;
mod config;

pub mod commands;
pub mod evaluate;
pub mod input;

const HISTORYSIZE: usize = 500;

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
    history: VecDeque<String>,
    alias: HashMap<String, Vec<String>>,
    stdout: bool,
}

impl Values {
    pub fn new() -> Self{
        Values {
            dir: env::current_dir().unwrap(),
            args: None,
            vars: HashMap::new(),
            pipe: None,
            history: VecDeque::new(),
            alias: HashMap::new(),
            stdout: true,
        }
    }
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

    let command = commands::search(&c, values);
    match command {
        Some(x) => (x.run(values), Some(c)),
        None => (vec![Err(format!("Unknown command: {}", c))], None),
    }
}

fn run_arg(arg: String, values: &mut Values){
    values.args = Some(vec![arg.clone()]);
    let cat = commands::search("cat", values);
    let result = cat.unwrap().run(values)[0].clone();
    if result.is_err(){
        eprintln!("rsh: {}",result.err().unwrap());
        return;
    }
    let mut s = result.unwrap();

    s = s.lines().filter(|l| !l.trim().starts_with("#")).collect();

    let (result, command) = main_loop(values, s.trim());

    for r in result {
        match r {
            Ok(x) => {
                print!("{}", x);
            }
            Err(x) => {
                match command {
                    None => eprint!("{}", x),
                    Some(_) => eprint!("{}: {}", command.clone().unwrap(), x),
                }
            }
        }
    }
    values.args = None;
}

fn main() {
    let mut values: Values = Values::new();
    let mut color = "\x1b[35m";
    values.history = get_history();
    let mut args = env::args().into_iter();
    args.next();
    for arg in args{
        run_arg(arg, &mut values);
    }
    if env::args().len() > 1 {
        return;
    }
    loop {
        io::Write::flush(&mut io::stdout()).expect("flush failed!");
        print!(
            "\x1b[34m{}\n{}> \x1b[39m",
            values.dir.to_string_lossy(),
            color
        );
        let s = input::input(values.history.clone());
        if s == "\n" || s == String::new() {
            continue;
        }
        values.history.push_back(s.clone());
        store_history(values.history.clone());
        if values.history.len() > HISTORYSIZE{
            values.history.pop_front();
        }
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
