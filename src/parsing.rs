use crate::VarTypes;
use std::collections::{HashMap, VecDeque};

type Operations = fn(VarTypes, VarTypes) -> VarTypes;

pub fn parse_arg(s: &str, vars: &HashMap<String, VarTypes>) -> Result<Vec<String>, &'static str> {
    let mut s_ = String::from(s);
    let mut result: Vec<String> = Vec::new();
    let mut slices = String::new();
    let mut in_crl: Vec<String> = Vec::new();
    let mut dq: bool = false; //check if in double quoute
    let mut sq: bool = false; //check if in single quoute
    let mut crl: usize = 0; //check if in curly and which layer
    s_.push('\n');
    loop {
        for c in s_.chars() {
            if c == '\"' && !sq {
                dq = !dq;
                continue;
            }
            if c == '\'' && !dq {
                sq = !sq;
                continue;
            }
            if c == '{' && !sq {
                in_crl.push(String::new());
                crl += 1;
                continue;
            }
            if c == '}' && !sq && crl > 0 {
                crl -= 1;
                let eval_res = evaluate(&in_crl[crl], vars);
                if crl > 0 {
                    in_crl[crl - 1] = eval_res;
                    continue;
                }
                slices = format!("{}{}", slices, eval_res);
                continue;
            }
            if crl > 0 {
                in_crl[crl - 1].push(c);
                continue;
            }
            if c == ' ' && !dq && !sq {
                result.push(slices.clone());
                slices = String::new();
                continue;
            }
            slices.push(c);
        }
        if dq || sq {
            print!("> ");
            s_ = crate::input();
            continue;
        }
        if crl > 0 {
            return Err("curly brace opened but never closed: expected }");
        }
        slices.pop();
        result.push(slices.clone());
        break;
    }
    return Ok(result);
}

fn find_var(s: &str, vars: &HashMap<String, VarTypes>) -> VarTypes {
    match s.parse::<i32>() {
        Ok(x) => return VarTypes::I(x),
        Err(_) => (),
    }
    if !vars.contains_key(s) {
        return VarTypes::N;
    }
    return vars.get(s).unwrap().clone();
}

fn add(var1: VarTypes, var2: VarTypes) -> VarTypes {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    if var1_type == 'N' && var2_type == 'N' {
        return VarTypes::N;
    }
    if var1_type == 'N' {
        return var2;
    }
    if var2_type == 'N' {
        return var1;
    }
    if var1_type == 'I' && var2_type == 'I' {
        return VarTypes::I(var1.get_i() + var2.get_i());
    }
    VarTypes::S(var1.get_s() + &var2.get_s())
}

fn sub(var1: VarTypes, var2: VarTypes) -> VarTypes {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    if var1_type == 'I' && var2_type == 'I' {
        return VarTypes::I(var1.get_i() - var2.get_i());
    }
    VarTypes::N
}

fn divi(var1: VarTypes, var2: VarTypes) -> VarTypes {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    if var1_type == 'I' && var2_type == 'I' {
        return VarTypes::I(var1.get_i() / var2.get_i());
    }
    VarTypes::N
}

fn mult(var1: VarTypes, var2: VarTypes) -> VarTypes {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    if var1_type == 'I' && var2_type == 'I' {
        return VarTypes::I(var1.get_i() * var2.get_i());
    }
    VarTypes::N
}

fn pow(var1: VarTypes, var2: VarTypes) -> VarTypes {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    if var1_type == 'I' && var2_type == 'I' {
        if var2.get_i() < 0 {
            return VarTypes::I(1 / var1.get_i().pow((-var2.get_i()) as u32));
        }
        return VarTypes::I(var1.get_i().pow(var2.get_i() as u32));
    }
    VarTypes::N
}

fn evaluate(s: &str, vars: &HashMap<String, VarTypes>) -> String {
    let mut vals: VecDeque<String> = VecDeque::from([String::new()]);
    let mut curr = 0;
    let mut operations: HashMap<char, Operations> = HashMap::new();
    operations.insert('+', add);
    operations.insert('-', sub);
    operations.insert('/', divi);
    operations.insert('*', mult);
    operations.insert('^', pow);
    for c in s.chars() {
        if operations.contains_key(&c) {
            vals.push_back(c.to_string());
            vals.push_back(String::new());
            curr += 2;
            continue;
        }
        vals[curr].push(c);
    }
    if vals.len() == 0 {
        return String::new();
    }
    if vals.len() == 1 {
        return match find_var(s, vars) {
            VarTypes::I(x) => x.to_string(),
            VarTypes::S(x) => x,
            VarTypes::N => String::new(),
        };
    }
    let temp = vals.pop_front().unwrap();
    let mut result = match find_var(&temp, vars) {
        VarTypes::N => VarTypes::S(temp),
        x => x,
    };
    while vals.len() > 0 {
        let operant = vals.pop_front().unwrap().chars().next().unwrap();
        let temp = vals.pop_front().unwrap();
        let var = match find_var(&temp, vars) {
            VarTypes::N => VarTypes::S(temp),
            x => x,
        };
        result = operations.get(&operant).unwrap()(result.clone(), var);
    }
    return match result {
        VarTypes::I(x) => x.to_string(),
        VarTypes::S(x) => x,
        VarTypes::N => String::new(),
    };
}
