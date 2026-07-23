use crate::{CmdVals, normalise_dir, run_queue};
use crate::{Values, VarTypes};
use std::collections::VecDeque;
use std::{env, vec};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

pub enum Commands {
    Unknown(String),
    Exit,
    Echo,
    Ls,
    Cd,
    Pwd,
    Let,
    Touch,
    Cat,
    Mkdir,
    Write,
    Alias,
    If
}

#[derive(Clone,Debug)]
pub struct ErrType {
    /// Parsing error: 422
    /// Notfound : 404
    /// Invalid args: 400
    code: u32,
    message: String
}

impl ErrType{
    pub fn new(c:u32, m:String) -> Self{
        Self { code: c, message: m }
    }
    pub fn message(&self) -> String{
        self.message.clone()
    }
    fn format(&mut self, command: &str){
        match self.code {
            404 => self.message = format!("{}: {}\n", self.message(), command),
            _ => self.message = format!("{}: {}\n", command, self.message())
        }
    }
}

pub fn search(command: String) -> Commands {
    use Commands::*;
    match command.as_str() {
        "exit" => Exit,
        "echo" => Echo,
        "ls" => Ls,
        "cd" => Cd,
        "pwd" => Pwd,
        "let" => Let,
        "touch" => Touch,
        "cat" => Cat,
        "mkdir" => Mkdir,
        "write" => Write,
        "alias" => Alias,
        "if" => If,
        _ => Unknown(command),
    }
}

impl Commands {
    pub fn run(&self, value: &mut Values) -> Vec<Result<String, ErrType>> {
        use Commands::*;
        let mut r = match self {
            Unknown(command) => try_run(command, value),
            Exit => exit(),
            Echo => echo(value),
            Ls => ls(value),
            Cd => cd(value),
            Pwd => pwd(value),
            Let => let_(value),
            Touch => touch(value),
            Cat => cat(value),
            Mkdir => mkdir(value),
            Write => write(value),
            Alias => alias(value),
            If => if_(value)
        };
        for i in 0..r.len(){
            match &r[i] {
                Ok(_) => {},
                Err(e) => {
                    let mut err = e.clone();
                    err.format(&value.cur_com.command);
                    r[i] = Err(err);
                }
            }
        }
        r
    }
}

fn execute(mut s: process::Child, value: &mut Values) -> Vec<Result<String, ErrType>> {
    if !value.cur_com.pipe.is_none() {
        let mut stdin = s.stdin.take().unwrap();
        stdin
            .write_all(value.cur_com.pipe.clone().unwrap().as_bytes())
            .unwrap();
    }
    let mut x = String::new();
    s.wait().expect("Cannot run command");
    if !s.stderr.is_none() {
        _ = s.stderr.unwrap().read_to_string(&mut x);
        if !x.is_empty() {
            return vec![Err(ErrType{code: 400,message: x})];
        }
    }
    if s.stdout.is_none() {
        return vec![Ok(String::new())];
    }
    s.stdout
        .unwrap()
        .read_to_string(&mut x)
        .expect("Cannot open file");
    return vec![Ok(x)]
}

fn try_run(command: &str, value: &mut Values) -> Vec<Result<String, ErrType>> {
    if command == "" {
        return vec![Ok(String::new())];
    }
    let mut c = Command::new(command);
    let result;
    if !value.cur_com.args.is_none() {
        let args = value.cur_com.args.clone().unwrap();
        c.args(args);
    }
    if !value.cur_com.pipe.is_none() {
        c.stdin(Stdio::piped());
    }
    if !value.cur_com.stdout {
        c.stdout(Stdio::piped());
    }
    c.stderr(Stdio::piped());
    result = c.spawn();
    match result {
        Ok(s) => {
            return execute(s,value);
        }
        Err(_) => {
            let temp = push_dir(command, &value.dir);
            if dir_exists(&temp) == 0 {
                let mut exec = Command::new(temp);
                let exec_result = exec.spawn();
                match exec_result {
                    Ok(s) => {
                        return execute(s,value);
                    },
                    Err(_) => return vec![Err(ErrType{
                        code: 400,
                        message: format!("Not an executable")
                    })],
                }
            }
            vec![Err(ErrType{
                code: 404,
                message: format!("Unknown command")}
            )]
        }
    }
}

fn exit() -> Vec<Result<String, ErrType>> {
    process::exit(1);
}

fn echo(value: &mut Values) -> Vec<Result<String, ErrType>> {
    if value.cur_com.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args_ = value.cur_com.args.clone().unwrap();
    let mut result = String::new();
    for s in args_ {
        result = format!("{}{}\n", result, s);
    }
    vec![Ok(result)]
}

///return
/// - -1 if not exist
/// - 1 if is dir
/// - 0 if is file
fn dir_exists(dir: &PathBuf) -> i32 {
    if !fs::exists(dir).expect(&format!(
        "Can't check existence of file {}",
        dir.to_string_lossy()
    )) {
        return -1;
    }
    if fs::metadata(dir).unwrap().is_dir() {
        return 1;
    }
    return 0;
}

fn push_dir(arg: &str, dir: &PathBuf) -> PathBuf {
    let mut dir_ = dir.clone();
    let mut arg_ = arg.chars();
    if arg_.next().unwrap() == '~' {
        dir_.push(PathBuf::from(format!(
            "{}{}",
            env::home_dir().unwrap().to_string_lossy(),
            arg_.as_str()
        )));
        return dir_;
    }
    dir_.push(PathBuf::from(arg));
    dir_
}

fn cd(value: &mut Values) -> Vec<Result<String, ErrType>> {
    if value.cur_com.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = value.cur_com.args.clone().unwrap();
    if args.len() > 1 {
        return vec![Err(ErrType{
            code: 400,
            message: format!("too many arguments")
        })];
    }
    let arg = &args[0].trim();
    let dir = push_dir(arg, &value.dir);
    match dir_exists(&dir) {
        -1 => {
            return vec![Err(ErrType{
                code: 400,
                message: format!("cannot access {}: No such file or directory",arg)
            })];
        }
        0 => {
            return vec![Err(ErrType{
                code: 400,
                message: format!("{}: Not a directory", arg)
            })];
        }
        _ => {}
    }
    value.dir = normalise_dir(&dir);
    vec![Ok(String::new())]
}

fn ls(value: &mut Values) -> Vec<Result<String, ErrType>> {
    let mut result: Vec<Result<String, ErrType>> = Vec::new();
    if value.cur_com.args.is_none() {
        let mut s = String::new();
        let paths = fs::read_dir(&value.dir).unwrap();

        for path in paths {
            s += path.unwrap().file_name().to_str().unwrap();
            s.push('\n');
        }
        result.push(Ok(s));
        return result;
    }
    let args = value.cur_com.args.clone().unwrap();
    let check = args.len() > 1;
    let mut s = String::new();
    for arg in args {
        match dir_exists(&push_dir(&arg, &value.dir)) {
            -1 => {
                result.push(Err(ErrType{
                    code: 400,
                    message: format!("cannot access {}: No such file or directory",arg)
                }));
                continue;
            }
            0 => {
                result.push(Ok(arg + "\n"));
                continue;
            }
            _ => {}
        }
        if check {
            s += &format!("{}:\n", arg);
        }
        let dir_ = push_dir(&arg, &value.dir);
        let paths = fs::read_dir(dir_).unwrap();

        for path in paths {
            if check {
                s += "  ";
            }
            s += path.unwrap().file_name().to_str().unwrap();
            s.push('\n');
        }
    }
    if s.is_empty() {
        return result;
    }
    result.push(Ok(s));
    result
}

fn pwd(value: &mut Values) -> Vec<Result<String, ErrType>> {
    return vec![Ok(String::from(value.dir.to_str().unwrap()) + "\n")];
}

fn let_(value: &mut Values) -> Vec<Result<String, ErrType>> {
    if value.cur_com.args.is_none() {
        return vec![Err(ErrType{
            code: 400,
            message: format!("expect variable name")
        })];
    }
    let args = value.cur_com.args.clone().unwrap();
    if args.len() > 1 {
        return vec![Err(ErrType{
            code: 400,
            message: format!("too many arguments")
        })];
    }
    let mut var_name = String::new();
    let mut var_val = String::new();
    let mut found_eq = false;
    for c in args[0].chars() {
        if found_eq {
            var_val.push(c);
            continue;
        }
        if c == '=' {
            found_eq = true;
            continue;
        }
        var_name.push(c);
    }
    if var_name.parse::<i32>().is_ok() {
        return vec![Err(ErrType{
            code: 400,
            message: format!("{} is not a valid name", var_name)
        })];
    }
    if !found_eq {
        value.vars.insert(args[0].clone(), VarTypes::N);
        return vec![Ok(String::new())];
    }

    match var_val.parse::<bool>() {
        Ok(x) => _ = value.vars.insert(var_name.clone(), VarTypes::B(x)),
        Err(_) => _ = (),
    }

    match var_val.parse::<i32>() {
        Ok(x) => _ = value.vars.insert(var_name.clone(), VarTypes::I(x)),
        Err(_) => _ = (),
    }

    match var_val.parse::<f32>() {
        Ok(x) => _ = value.vars.insert(var_name, VarTypes::F(x)),
        Err(_) => _ = value.vars.insert(var_name, VarTypes::S(var_val)),
    }

    vec![Ok(String::new())]
}

fn touch(value: &mut Values) -> Vec<Result<String, ErrType>> {
    if value.cur_com.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = value.cur_com.args.clone().unwrap();
    let mut result: Vec<Result<String, ErrType>> = Vec::new();
    for i in 0..args.len() {
        let dir = push_dir(&args[i], &value.dir);
        if dir_exists(&dir) == 0{
            continue;
        }
        let x = File::create(dir);
        if x.is_err() {
            result.push(Err(ErrType{
                code: 400,
                message: format!("can not create file {}", args[i])
            }));
            result.push(Err(ErrType{
                code: 400,
                message: format!("can not create file {}", args[i])
            }));
        }
    }
    result
}

fn cat(value: &mut Values) -> Vec<Result<String, ErrType>> {
    let mut result: Vec<Result<String, ErrType>> = Vec::new();
    if value.cur_com.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = value.cur_com.args.clone().unwrap();
    let check = args.len() > 1;
    let mut s = String::new();
    for arg in args {
        match dir_exists(&push_dir(&arg, &value.dir)) {
            -1 => {
                result.push(Err(ErrType{
                    code: 400,
                    message: format!("cannot access {}: No such file or directory",arg)
                }));
                continue;
            }
            1 => {
                result.push(Err(ErrType{
                    code: 400,
                    message: format!("cannot read {} is a directory", arg)
                    }));
                continue;
            }
            _ => {}
        }
        if check {
            s += &format!("{}:\n", arg);
        }
        let dir_ = push_dir(&arg, &value.dir);
        let mut file = File::open(dir_).unwrap();
        let mut contents = String::new();

        let handler = file.read_to_string(&mut contents);
        if handler.is_err() {
            result.push(Err(ErrType{
                code: 400,
                message: format!("failed to read: {}", arg)
            }));
            continue;
        }
        s += &contents;
    }
    if s.is_empty() {
        return result;
    }
    result.push(Ok(s));
    result
}

fn mkdir(value: &mut Values) -> Vec<Result<String, ErrType>> {
    let mut result: Vec<Result<String, ErrType>> = Vec::new();
    if value.cur_com.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = value.cur_com.args.clone().unwrap();
    for arg in args {
        let mut temp = push_dir(&arg, &value.dir);
        temp.pop();
        match dir_exists(&temp) {
            1 => {}
            _ => {
                result.push(Err(ErrType{
                    code: 400,
                    message: format!("directory {} does not exist",temp.to_string_lossy()
                )
                }));
                continue;
            }
        }
        let dir_ = push_dir(&arg, &value.dir);
        let handler = fs::create_dir(dir_);
        if handler.is_err() {
            result.push(Err(ErrType{
                code: 400,
                message: format!("failed to create: {}", arg)
            }));
            continue;
        }
    }
    result
}

fn write(value: &mut Values) -> Vec<Result<String, ErrType>> {
    if value.cur_com.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = value.cur_com.args.clone().unwrap();
    if value.cur_com.pipe.is_none() {
        return vec![Err(ErrType{
            code: 400,
            message: format!("cannot write")
        })];
    }
    if args.len() == 1 {
        let dir_ = push_dir(&args[0], &value.dir);
        let mut parent = dir_.clone();
        parent.pop();
        match dir_exists(&parent) {
            1 => {}
            _ => {
                return vec![Err(ErrType{
                    code: 400,
                    message: format!("cannot access {}: No such file or directory",args[0])
                })];
            }
        }
        let mut file = File::create(dir_).unwrap();
        _ = file.write_all(value.cur_com.pipe.clone().unwrap().as_bytes());
        return vec![Ok(String::new())];
    }
    let mut result: Vec<Result<String, ErrType>> = Vec::new();
    let mut flaged = false;
    for arg in args {
        if arg.as_bytes()[0] == b'-' {
            flaged = true;
            continue;
        }
        let dir_ = push_dir(&arg, &value.dir);
        match dir_exists(&dir_) {
            0 => {}
            _ => {
                result.push(Err(ErrType{
                    code: 400,
                    message: format!("cannot access {}: No such file or directory",arg)
                }));
            }
        }
        let mut data = value.cur_com.pipe.clone().unwrap();
        if flaged {
            let mut file = File::open(dir_.clone()).unwrap();
            let mut contents = String::new();
            let handler = file.read_to_string(&mut contents);
            if handler.is_err() {
                result.push(Err(ErrType{
                    code: 400,
                    message: format!("failed to read: {}", arg)
                }));
                continue;
            }
            data += &contents;
        }
        let mut file = File::create(dir_).unwrap();
        _ = file.write_all(data.as_bytes());
    }
    result
}

fn alias(value: &mut Values) -> Vec<Result<String, ErrType>>{
    if value.cur_com.args.is_none() {
        return vec![Err(ErrType{
            code: 400,
            message: format!("expect alias")
        })];
    }
    let args = value.cur_com.args.clone().unwrap();
    if args.len() > 1 {
        return vec![Err(ErrType{
            code: 400,
            message: format!("too many arguments")
        })];
    }
    let mut var_name = String::new();
    let mut var_val = String::new();
    let mut found_eq = false;
    for c in args[0].chars() {
        if found_eq {
            var_val.push(c);
            continue;
        }
        if c == '=' {
            found_eq = true;
            continue;
        }
        var_name.push(c);
    }
    if var_name.parse::<i32>().is_ok() {
        return vec![Err(ErrType{
            code: 400,
            message: format!("{} is not a valid name", var_name)
        })];
    }
    if !found_eq {
        return vec![Err(ErrType{
            code: 400,
            message: format!("expect value")
        })];
    }
    value.alias.insert(var_name, var_val);

    vec![Ok(String::new())]
}

fn if_(value: &mut Values) -> Vec<Result<String,ErrType>>{
    if value.cur_com.args.is_none(){
        while !(value.com_q.is_empty() || value.get_com() == "end"){}
        return vec![Err(ErrType{
            code: 400,
            message: format!("expect argument")
        })];
    }
    let args = value.cur_com.args.clone().unwrap();
    let condition = args[0].clone().parse::<bool>();
    if condition.is_err(){
        while !(value.com_q.is_empty() || value.get_com() == "end"){}
        return vec![Err(ErrType{
            code: 400,
            message: format!("not a valid condition")
        })];
    }
    let condition = condition.unwrap();
    if !condition {
        while !value.com_q.is_empty(){
            if value.get_com() == "end"{
                return vec![Ok(String::new())];
            }
        }
        return vec![Err(ErrType{
            code: 400,
            message: format!("could not found end")
        })];
    }else{
        let mut value_temp = value.clone();
        value_temp.cur_com = CmdVals::new();
        value_temp.com_q = VecDeque::new();
        while !value.com_q.is_empty(){
            let com = value.get_com();
            if com == "end"{
                return run_queue(&mut value_temp);
            }
            value_temp.com_q.push_back(com);
        }
    }
    vec![Err(ErrType{
        code: 400,
        message: format!("could not found end")
    })]
}

// fn while(value: &mut Values) -> Vec<Result<String,String>>{

// }
