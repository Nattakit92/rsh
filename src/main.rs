use std::collections::{HashMap, VecDeque};
use std::env::Args;
use std::path::PathBuf;
use std::{env, io};

use crate::config::{get_history, run_startup, store_history};

mod config;

pub mod parsing;
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
    alias: HashMap<String, String>,
    stdout: bool,
    arg_extend: bool,
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
            arg_extend: false,
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

pub fn main_loop(values: &mut Values, s: &str) -> (Vec<Result<String, String>>, Option<String>) {
    if s.is_empty() {
        return (vec![], None);
    }
    let args = parsing::parse_arg(s, values);
    let mut c: String;
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
    let alias;
    if values.alias.contains_key(&c){
        if values.args.is_some(){
            let args = values.args.clone().unwrap();
            let mut i = 0;
            for arg in args{
                match arg.parse::<i32>() {
                    Ok(x) => _ = values.vars.insert(i.to_string(), VarTypes::I(x)),
                    Err(_) => _ = values.vars.insert(i.to_string(), VarTypes::S(arg)),
                }
                i += 1;
            }
        }
        values.arg_extend = true;
        alias = crate::parsing::parse_arg(&values.alias[&c].clone(), values).unwrap();
        if values.alias[&c].len() > 1 && values.arg_extend{
            let mut temp = Vec::from(&alias[1..]);
            if let Some(args) = values.args.clone(){
                temp.extend(args);
            }
            values.args = Some(temp);
        }
        c = alias[0].clone();
    }

    let command = commands::search(&c);
    match command {
        Some(x) => (x.run(values), Some(c)),
        None => (vec![Err(format!("Unknown command: {}", c))], None),
    }
}

fn run_arg(arg: String, values: &mut Values, args: Args){
    values.args = Some(vec![arg.clone()]);
    let cat = commands::search("cat");
    let result = cat.unwrap().run(values)[0].clone();
    if result.is_err(){
        return;
    }
    let mut s = result.unwrap();

    let mut i = 0;
    for arg in args{
        match arg.parse::<i32>() {
            Ok(x) => _ = values.vars.insert(i.to_string(), VarTypes::I(x)),
            Err(_) => _ = values.vars.insert(i.to_string(), VarTypes::S(arg)),
        }
        i += 1;
    }

    s = s.lines()
        .filter(|l| !l.trim().starts_with("#"))
        .map(|l| format!("{}\n", l))
        .collect();

    let (result, _) = main_loop(values, s.trim());

    for r in result {
        match r {
            Ok(x) => {
                print!("{}", x);
            }
            Err(_) => {}
        }
    }
    values.args = None;
}

fn main() {
    let mut values: Values = Values::new();
    let temp = values.dir.clone();
    let mut color = "\x1b[35m";
    values.history = get_history();
    let mut args = env::args().into_iter();
    args.next();
    if let Some(arg) = args.next(){
        run_arg(arg, &mut values, args);
    }

    if env::args().len() > 1 {
        return;
    }
    run_startup(&mut values);
    values.dir = temp;
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
        values.arg_extend = false;
        let mut i = 0;
        while values.vars.contains_key(&i.to_string()){
            values.vars.remove(&i.to_string());
            i+=1;
        }
    }
}
