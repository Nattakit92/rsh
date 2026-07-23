use std::collections::{HashMap, VecDeque};
use std::env::Args;
use std::path::PathBuf;
use std::{env, io};

use crate::commands::ErrType;
use crate::config::{get_history, run_startup, store_history};
use crate::parsing::{parse_arg, parse_commands};

mod config;
pub mod parsing;
pub mod commands;
pub mod evaluate;
pub mod input;

const HISTORYSIZE: usize = 500;

#[derive(Clone,PartialEq)]
pub enum VarTypes {
    ///int type
    I(i32),
    ///string type
    S(String),
    /// bool type
    B(bool),
    /// none type
    N,
}

impl VarTypes {
    pub fn set_i(val: String) -> VarTypes {
        Self::I(val.parse::<i32>().unwrap())
    }
    pub fn set_s(val: String) -> VarTypes {
        Self::S(val)
    }
    pub fn set_b(val: String) -> VarTypes {
        Self::B(val.parse::<bool>().unwrap())
    }
    pub fn get_type(val: String) -> VarTypes {
        if val.parse::<i32>().is_ok() {
            return Self::I(0);
        }
        if val.parse::<bool>().is_ok(){
            return Self::B(false);
        }
        Self::S(val)
    }
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
            Self::B(x) => x.to_string(),
            Self::N => String::new(),
        }
    }
}
#[derive(Clone)]
pub struct Values {
    dir: PathBuf,
    vars: HashMap<String, VarTypes>,
    history: VecDeque<String>,
    alias: HashMap<String, String>,
    com_q: VecDeque<String>,
    cur_com: CmdVals
}

impl Values{
    pub fn new() -> Self {
        Values {
            dir: env::current_dir().unwrap(),
            vars: HashMap::new(),
            history: VecDeque::new(),
            alias: HashMap::new(),
            com_q: VecDeque::new(),
            cur_com: CmdVals::new()
        }
    }
    pub fn get_com(&mut self) -> String{
        self.com_q.pop_front().unwrap()
    }
}

#[derive(Clone)]
pub struct CmdVals {
    command: String,
    args: Option<Vec<String>>,
    pipe: Option<String>,
    stdout: bool,
}

impl CmdVals {
    pub fn new() -> Self{
        CmdVals {
            command: String::new(),
            args: None,
            pipe: None,
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

pub fn main_loop(values: &mut Values, s: &str) -> Vec<Result<String, ErrType>> {
    if s.is_empty() {
        return vec![];
    }

    //parse and put command into queue
    match parse_commands(s, values) {
        Err(e) => return vec![Err(ErrType::new(422, format!("{}", e)))],
        _ => ()
    }

    run_queue(values)
}

///use to run command in values.com_q
fn run_queue(values: &mut Values) -> Vec<Result<String, ErrType>>{
    let mut result: Vec<Result<String, ErrType>> = vec![];

    while !values.com_q.is_empty(){

        values.cur_com = CmdVals::new();

        //parse arguement
        match parse_arg(values){
            Ok(mut args) => {
                if args.is_empty(){
                    return vec![Ok(String::new())]
                }
                values.cur_com.command = args.remove(0);
                if args.is_empty() {
                    values.cur_com.args = None;
                } else{
                    values.cur_com.args = Some(args);
                }
            }
            Err(e) => {
                return vec![Err(ErrType::new(422, format!("{}", e)))];
            }
        }

        //check for alias
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

        //search and run command
        let command = commands::search(values.cur_com.command.clone());
        let r = command.run(values);
        result.extend(r);
    }
    result
}

fn run_arg(arg: String, values: &mut Values, args: Args){
    values.cur_com.command = String::from("cat");
    values.cur_com.args = Some(vec![arg.clone()]);
    let cat = commands::search(String::from("cat"));
    let result = cat.run(values)[0].clone();
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

    let result = main_loop(values, s.trim());

    for r in result {
        match r {
            Ok(x) => {
                print!("{}", x);
            }
            Err(_) => {}
        }
    }
    values.cur_com = CmdVals::new();
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
        let result = main_loop(&mut values, s.trim());

        for r in result {
            match r {
                Ok(x) => {
                    print!("{}", x);
                    color = "\x1b[35m";
                }
                Err(x) => {
                    eprint!("{}", x.message());
                    color = "\x1b[31m";
                }
            }
        }
        values.cur_com = CmdVals::new();
        env::set_current_dir(&values.dir).expect("Invalid location");
        let mut i = 0;
        while values.vars.contains_key(&i.to_string()){
            values.vars.remove(&i.to_string());
            i+=1;
        }
    }
}
