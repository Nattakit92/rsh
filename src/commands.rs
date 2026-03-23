use std::fs;
use std::path::PathBuf;
use std::process;

pub enum Commands {
    Exit,
    Echo,
    Ls,
    Cd,
    Pwd,
}

pub fn search(t: &Vec<&str>) -> Option<Commands> {
    match t[0] {
        "exit" => Some(Commands::Exit),
        "echo" => Some(Commands::Echo),
        "ls" => Some(Commands::Ls),
        "cd" => Some(Commands::Cd),
        "pwd" => Some(Commands::Pwd),
        _ => None,
    }
}

impl Commands {
    pub fn run(&self, t: &Vec<&str>, dir: &PathBuf) -> Option<PathBuf> {
        let mut args: Option<&Vec<String>> = None;
        let temp;
        if t.len() > 1 {
            temp = parse_arg(t[1]);
            args = Some(&temp);
        }
        match self {
            Self::Exit => exit(),
            Self::Echo => echo(&args),
            Self::Ls => ls(&args, &dir),
            Self::Cd => cd(&args, &dir),
            Self::Pwd => pwd(&dir),
        }
    }
}

fn parse_arg(s: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut temp = String::new();
    let mut dq: i32 = -1;
    let mut sq: i32 = -1;
    let s_b = s.as_bytes();
    for i in 0..s.len() {
        if s_b[i] == b'\"' && sq == -1 {
            if dq == -1 {
                dq = i as i32;
                continue;
            }
            dq = -1;
            continue;
        }
        if s_b[i] == b'\'' && dq == -1 {
            if sq == -1 {
                sq = i as i32;
                continue;
            }
            sq = -1;
            continue;
        }
        if s_b[i] == b' ' && dq == -1 && sq == -1 {
            result.push(temp.clone());
            temp = String::new();
            continue;
        }
        temp.push(s_b[i] as char);
    }
    result.push(temp.clone());
    return result;
}

fn exit() -> Option<PathBuf> {
    process::exit(1);
}

fn echo(args: &Option<&Vec<String>>) -> Option<PathBuf> {
    if args.is_none() {
        println!();
        return None;
    }
    let args_ = args.clone().unwrap();
    for s in args_ {
        print!("{} ", s);
    }
    println!();
    return None;
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

fn push_dir(arg: &str, dir: &PathBuf) -> Option<PathBuf> {
    let mut dir_ = dir.clone();
    dir_.push(PathBuf::from(arg));
    Some(dir_)
}

fn cd(args: &Option<&Vec<String>>, dir: &PathBuf) -> Option<PathBuf> {
    if args.is_none() {
        println!();
        return None;
    }
    if args.unwrap().len() > 1 {
        eprintln!("cd: too many arguments");
        return None;
    }
    let arg = &args.unwrap()[0];
    let dir_ = push_dir(arg, dir).unwrap();
    match dir_exists(&dir_) {
        -1 => {
            eprintln!("cd: cannot access {}: No such file or directory", arg);
            return None;
        }
        0 => {
            eprintln!("cd: {}: Not a directory", arg);
            return None;
        }
        _ => {}
    }
    return Some(dir_);
}

fn ls(args: &Option<&Vec<String>>, dir: &PathBuf) -> Option<PathBuf> {
    if args.is_none() {
        let paths = fs::read_dir(dir).unwrap();

        for path in paths {
            println!("{}", path.unwrap().file_name().to_string_lossy());
        }
        return None;
    }
    for arg in args.unwrap() {
        match dir_exists(&push_dir(arg, dir).unwrap()) {
            -1 => {
                eprintln!("ls: cannot access {}: No such file or directory", arg);
                continue;
            }
            0 => {
                println!("{}", arg);
                continue;
            }
            _ => {}
        }
        let check = args.unwrap().len() > 1;
        if check {
            println!("{}:", arg);
        }
        let dir_ = cd(&Some(&vec![String::from(arg.clone())]), &dir).unwrap();
        let paths = fs::read_dir(dir_).unwrap();

        for path in paths {
            if check {
                print!("  ");
            }
            println!("{}", path.unwrap().file_name().to_string_lossy());
        }
    }
    return None;
}

fn pwd(dir: &PathBuf) -> Option<PathBuf> {
    println!("{}", dir.to_string_lossy());
    return None;
}
