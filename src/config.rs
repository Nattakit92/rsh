use crate::{Values, commands};
use std::collections::VecDeque;
use std::env;

pub fn get_history() -> VecDeque<String>{
    let mut values = Values::new();
    values.dir = env::home_dir().unwrap();
    values.args = Some(vec![String::from(".config"),String::from(".config/rsh")]);
    let mkdir = commands::search("mkdir", &mut values);
    mkdir.unwrap().run(&mut values);
    values.args = Some(vec![String::from(".config/rsh/history")]);
    let touch = commands::search("touch", &mut values);
    touch.unwrap().run(&mut values);
    let mut history: VecDeque<String> = VecDeque::from([String::new()]);
    let cat = commands::search("cat", &mut values);
    let temp = cat.unwrap().run(&mut values);
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
    history
}

pub fn store_history(history: VecDeque<String>){
    let mut values = Values::new();
    values.dir = env::home_dir().unwrap();
    values.args = Some(vec![String::from(".config/rsh/history")]);
    let write = commands::search("write", &mut values);
    values.pipe = Some(Vec::from(history).join("\t"));
    write.unwrap().run(&mut values);
}
