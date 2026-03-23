use std::env;
use std::io;
use std::path::PathBuf;
mod commands;

fn input() -> String {
    io::Write::flush(&mut io::stdout()).expect("flush failed!");
    let mut s = String::new();
    match io::stdin().read_line(&mut s) {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    };
    return s;
}

fn tokenizer(s: &str) -> Vec<&str> {
    match s.split_once(' ') {
        Some((a, b)) => return vec![a, b],
        None => return vec![s],
    }
}

fn normalise_dir(path: &PathBuf) -> PathBuf {
    let mut dir: PathBuf = PathBuf::new();
    for d in path {
        if d == ".." {
            dir.pop();
            continue;
        }
        if d != "." {
            dir.push(d);
        }
    }
    return dir;
}

fn main() {
    let mut dir = env::current_dir().unwrap();
    loop {
        print!("$ ");
        let s = input();
        if s == "\n" {
            continue;
        }
        let t: Vec<&str> = tokenizer(s.trim());
        if t.is_empty() {
            continue;
        }
        let command = commands::search(&t);
        let path = match command {
            Some(com) => com.run(&t, &dir),
            None => {
                println!("Unknown command: {}", t[0]);
                continue;
            }
        };
        if path == None {
            continue;
        }
        dir = normalise_dir(&path.unwrap());
    }
}
