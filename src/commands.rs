use crate::normalise_dir;
use crate::{Values, VarTypes};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

pub enum Commands {
    Exit,
    Echo,
    Ls,
    Cd,
    Pwd,
    Let,
}

pub fn search(t: &Vec<&str>) -> Option<Commands> {
    match t[0] {
        "exit" => Some(Commands::Exit),
        "echo" => Some(Commands::Echo),
        "ls" => Some(Commands::Ls),
        "cd" => Some(Commands::Cd),
        "pwd" => Some(Commands::Pwd),
        "let" => Some(Commands::Let),
        _ => None,
    }
}

impl Commands {
    pub fn run(&self, values: &mut Values) -> Vec<Result<String, String>> {
        match self {
            Self::Exit => exit(),
            Self::Echo => echo(values),
            Self::Ls => ls(values),
            Self::Cd => cd(values),
            Self::Pwd => pwd(values),
            Self::Let => let_(values),
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
        return vec![Err(String::from("expect variable name"))];
    }
    let args = values.args.clone().unwrap();
    if args.len() > 1 {
        return vec![Err(String::from("too many arguments"))];
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
        Ok(x) => {
            values.vars.insert(var_name, VarTypes::I(x));
            return vec![Ok(String::new())];
        }

        Err(_) => {
            values.vars.insert(var_name, VarTypes::S(var_val));
            return vec![Ok(String::new())];
        }
    }
}
