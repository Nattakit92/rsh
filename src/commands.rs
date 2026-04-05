use crate::normalise_dir;
use crate::{Values, VarTypes};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

pub enum Commands<'a> {
    Unknown(&'a str),
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
}

pub fn search<'a>(command: &'a str) -> Option<Commands<'a>> {
    use Commands::*;
    match command {
        "exit" => Some(Exit),
        "echo" => Some(Echo),
        "ls" => Some(Ls),
        "cd" => Some(Cd),
        "pwd" => Some(Pwd),
        "let" => Some(Let),
        "touch" => Some(Touch),
        "cat" => Some(Cat),
        "mkdir" => Some(Mkdir),
        "write" => Some(Write),
        _ => Some(Unknown(command)),
    }
}

impl<'a> Commands<'a> {
    pub fn run(&self, values: &mut Values) -> Vec<Result<String, String>> {
        use Commands::*;
        match self {
            Unknown(command) => try_run(command, values),
            Exit => exit(),
            Echo => echo(values),
            Ls => ls(values),
            Cd => cd(values),
            Pwd => pwd(values),
            Let => let_(values),
            Touch => touch(values),
            Cat => cat(values),
            Mkdir => mkdir(values),
            Write => write(values),
        }
    }
}

fn try_run(command: &str, values: &mut Values) -> Vec<Result<String, String>> {
    if command == "" {
        return vec![Ok(String::new())];
    }
    let mut c = Command::new(command);
    let result;
    if !values.args.is_none() {
        let args = values.args.clone().unwrap();
        c.args(args);
    }
    if !values.pipe.is_none() {
        c.stdin(Stdio::piped());
    }
    if !values.stdout {
        c.stdout(Stdio::piped());
    }
    c.stderr(Stdio::piped());
    result = c.spawn();
    match result {
        Ok(mut s) => {
            if !values.pipe.is_none() {
                let mut stdin = s.stdin.take().unwrap();
                stdin
                    .write_all(values.pipe.clone().unwrap().as_bytes())
                    .unwrap();
            }
            let mut x = String::new();
            s.wait().expect("Cannot run command");
            if !s.stderr.is_none() {
                _ = s.stderr.unwrap().read_to_string(&mut x);
                return vec![Err(x)];
            }
            if s.stdout.is_none() {
                return vec![Ok(String::new())];
            }
            s.stdout
                .unwrap()
                .read_to_string(&mut x)
                .expect("Cannot open file");
            vec![Ok(x)]
        }
        Err(_) => {
            vec![Err(String::from("Unknown command\n"))]
        }
    }
}

fn exit() -> Vec<Result<String, String>> {
    process::exit(1);
}

fn echo(values: &mut Values) -> Vec<Result<String, String>> {
    if values.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args_ = values.args.clone().unwrap();
    let mut result = String::new();
    for s in args_ {
        result = format!("{}{}\n", result, s);
    }
    return vec![Ok(result)];
}

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
        dir_.push(PathBuf::from(String::from(format!(
            "{}{}",
            env::home_dir().unwrap().to_string_lossy(),
            arg_.as_str()
        ))));
        return dir_;
    }
    dir_.push(PathBuf::from(arg));
    return dir_;
}

fn cd(values: &mut Values) -> Vec<Result<String, String>> {
    if values.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = values.args.clone().unwrap();
    if args.len() > 1 {
        return vec![Err(String::from("too many arguments"))];
    }
    let arg = &args[0];
    let dir = push_dir(arg, &values.dir);
    match dir_exists(&dir) {
        -1 => {
            return vec![Err(format!(
                "cannot access {}: No such file or directory\n",
                arg
            ))];
        }
        0 => {
            return vec![Err(format!("{}: Not a directory\n", arg))];
        }
        _ => {}
    }
    values.dir = normalise_dir(&dir);
    return vec![Ok(String::new())];
}

fn ls(values: &mut Values) -> Vec<Result<String, String>> {
    let mut result: Vec<Result<String, String>> = Vec::new();
    if values.args.is_none() {
        let mut s = String::new();
        let paths = fs::read_dir(&values.dir).unwrap();

        for path in paths {
            s += path.unwrap().file_name().to_str().unwrap();
            s.push('\n');
        }
        result.push(Ok(s));
        return result;
    }
    let args = values.args.clone().unwrap();
    let check = args.len() > 1;
    let mut s = String::new();
    for arg in args {
        match dir_exists(&push_dir(&arg, &values.dir)) {
            -1 => {
                result.push(Err(format!(
                    "cannot access {}: No such file or directory\n",
                    arg
                )));
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
        let dir_ = push_dir(&arg, &values.dir);
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
    return result;
}

fn pwd(values: &mut Values) -> Vec<Result<String, String>> {
    return vec![Ok(String::from(values.dir.to_str().unwrap()) + "\n")];
}

fn let_(values: &mut Values) -> Vec<Result<String, String>> {
    if values.args.is_none() {
        return vec![Err(String::from("expect variable name\n"))];
    }
    let args = values.args.clone().unwrap();
    if args.len() > 1 {
        return vec![Err(String::from("too many arguments\n"))];
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
        return vec![Err(format!("{} is not a valid name", var_name))];
    }
    if !found_eq {
        values.vars.insert(args[0].clone(), VarTypes::N);
        return vec![Ok(String::new())];
    }

    match var_val.parse::<i32>() {
        Ok(x) => _ = values.vars.insert(var_name, VarTypes::I(x)),
        Err(_) => _ = values.vars.insert(var_name, VarTypes::S(var_val)),
    }
    return vec![Ok(String::new())];
}

fn touch(values: &mut Values) -> Vec<Result<String, String>> {
    if values.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = values.args.clone().unwrap();
    let mut result: Vec<Result<String, String>> = Vec::new();
    for i in 0..args.len() {
        let x = File::create(push_dir(&args[i], &values.dir));
        if x.is_err() {
            result.push(Err(format!("can not create file {}\n", args[i])));
            result.push(Err(format!("can not create file {}\n", args[i])));
        }
    }
    return result;
}

fn cat(values: &mut Values) -> Vec<Result<String, String>> {
    let mut result: Vec<Result<String, String>> = Vec::new();
    if values.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = values.args.clone().unwrap();
    let check = args.len() > 1;
    let mut s = String::new();
    for arg in args {
        match dir_exists(&push_dir(&arg, &values.dir)) {
            -1 => {
                result.push(Err(format!(
                    "cannot access {}: No such file or directory\n",
                    arg
                )));
                continue;
            }
            1 => {
                result.push(Err(format!("cannot read {} is a directory\n", arg)));
                continue;
            }
            _ => {}
        }
        if check {
            s += &format!("{}:\n", arg);
        }
        let dir_ = push_dir(&arg, &values.dir);
        let mut file = File::open(dir_).unwrap();
        let mut contents = String::new();

        let handler = file.read_to_string(&mut contents);
        if handler.is_err() {
            result.push(Err(format!("failed to read: {}\n", arg)));
            continue;
        }
        s += &contents;
    }
    if s.is_empty() {
        return result;
    }
    result.push(Ok(s));
    return result;
}

fn mkdir(values: &mut Values) -> Vec<Result<String, String>> {
    let mut result: Vec<Result<String, String>> = Vec::new();
    if values.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = values.args.clone().unwrap();
    for arg in args {
        let mut temp = push_dir(&arg, &values.dir);
        temp.pop();
        match dir_exists(&temp) {
            1 => {}
            _ => {
                result.push(Err(format!(
                    "directory {} does not exist\n",
                    temp.to_string_lossy()
                )));
                continue;
            }
        }
        let dir_ = push_dir(&arg, &values.dir);
        let handler = fs::create_dir(dir_);
        if handler.is_err() {
            result.push(Err(format!("failed to create: {}\n", arg)));
            continue;
        }
    }
    return result;
}

fn write(values: &mut Values) -> Vec<Result<String, String>> {
    if values.args.is_none() {
        return vec![Ok(String::new())];
    }
    let args = values.args.clone().unwrap();
    if values.pipe.is_none() {
        return vec![Err(String::from("cannot write"))];
    }
    if args.len() == 1 {
        let dir_ = push_dir(&args[0], &values.dir);
        match dir_exists(&dir_) {
            0 => {}
            _ => {
                return vec![Err(format!(
                    "cannot access {}: No such file or directory\n",
                    args[0]
                ))];
            }
        }
        let mut file = File::create(dir_).unwrap();
        _ = file.write_all(values.pipe.clone().unwrap().as_bytes());
        return vec![Ok(String::new())];
    }
    let mut result: Vec<Result<String, String>> = Vec::new();
    let mut flaged = false;
    for arg in args {
        if arg.as_bytes()[0] == b'-' {
            flaged = true;
            continue;
        }
        let dir_ = push_dir(&arg, &values.dir);
        match dir_exists(&dir_) {
            0 => {}
            _ => {
                result.push(Err(format!(
                    "cannot access {}: No such file or directory\n",
                    arg
                )));
            }
        }
        let mut data = values.pipe.clone().unwrap();
        if flaged {
            let mut file = File::open(dir_.clone()).unwrap();
            let mut contents = String::new();
            let handler = file.read_to_string(&mut contents);
            if handler.is_err() {
                result.push(Err(format!("failed to read: {}\n", arg)));
                continue;
            }
            data += &contents;
        }
        let mut file = File::create(dir_).unwrap();
        _ = file.write_all(data.as_bytes());
    }
    return result;
}
