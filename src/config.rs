use crate::{CmdVals, Values, commands};
use std::collections::VecDeque;
use std::env;

pub fn get_history() -> VecDeque<String>{
    let mut values = Values::new();
    values.dir = env::home_dir().unwrap();
    values.cur_com.args = Some(vec![String::from(".config"), String::from(".config/rsh")]);

    let mkdir = commands::search(String::from("mkdir"));
    mkdir.run(&mut values);
    values.cur_com.args = Some(vec![String::from(".config/rsh/history")]);

    let touch = commands::search(String::from("touch"));
    touch.run(&mut values);

    let mut history: VecDeque<String> = VecDeque::from([String::new()]);
    let cat = commands::search(String::from("cat"));
    let temp = cat.run(&mut values);

    if temp.is_empty(){
        return VecDeque::new();
    }
    let temp = temp[0].clone().unwrap();
    let mut i = 0;
    for c in temp.chars(){
        if c == '\t'{
            history.push_back(String::new());
            i += 1;
            continue;
        }
        history[i].push(c);
    }
    values.cur_com = CmdVals::new();
    history
}

pub fn store_history(history: VecDeque<String>){
    let mut values = Values::new();
    values.dir = env::home_dir().unwrap();
    values.cur_com.args = Some(vec![String::from(".config/rsh/history")]);

    let write = commands::search(String::from("write"));
    values.cur_com.pipe = Some(Vec::from(history).join("\t"));

    write.run(&mut values);
    values.cur_com = CmdVals::new();
}

pub fn run_startup(values: &mut Values){
    values.dir = env::home_dir().unwrap();
    values.cur_com.args = Some(vec![String::from(".config/rsh/rsh.rsh")]);
    let cat = commands::search(String::from("cat"));
    let result = cat.run(values)[0].clone();
    values.cur_com = CmdVals::new();
    if result.is_err(){
        return;
    }
    let mut s = result.unwrap();

    s = s.lines()
        .filter(|l| !l.trim().starts_with("#"))
        .map(|l| format!("{}\n", l))
        .collect();

    let result = crate::main_loop(values, s.trim());

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
