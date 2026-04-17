use crate::{Values, VarTypes};
use std::collections::{HashMap, VecDeque};

type Operations = fn(VarTypes, VarTypes) -> VarTypes;
type Comparision = fn(VarTypes, VarTypes) -> bool;

enum StateCompare {
    Normal,
    ExclamationMark,
    Equal,
    Greater,
    Less,
}

fn find_var(s: &str, values: &mut Values) -> VarTypes {
    let vars = &values.vars;
    if vars.contains_key(s) {
        values.arg_extend = false;
        return vars.get(s).unwrap().clone();
    }
    match s.parse::<i32>() {
        Ok(x) => {
            return VarTypes::I(x)
        },
        Err(_) => (),
    }
    VarTypes::N
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

fn equal(var1: VarTypes, var2: VarTypes) -> bool {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    if var1_type != var2_type {
        return false;
    }
    match var1_type {
        'I' => var1.get_i() == var2.get_i(),
        'S' => var1.get_s() == var2.get_s(),
        _ => true,
    }
}

fn isint(var1: &VarTypes, var2: &VarTypes) -> bool {
    let var1_type = var1.get_type();
    let var2_type = var2.get_type();
    var1_type == 'I' || var2_type == 'I'
}

fn inequal(var1: VarTypes, var2: VarTypes) -> bool {
    !equal(var1, var2)
}

fn greater(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(&var1, &var2) {
        return false;
    }
    return var1.get_i() > var2.get_i();
}

fn greaterequal(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(&var1, &var2) {
        return false;
    }
    if equal(var1.clone(), var2.clone()) {
        return true;
    }
    var1.get_i() > var2.get_i()
}

fn less(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(&var1, &var2) {
        return false;
    }
    return var1.get_i() < var2.get_i();
}

fn lessequal(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(&var1, &var2) {
        return false;
    }
    if equal(var1.clone(), var2.clone()) {
        return true;
    }
    var1.get_i() < var2.get_i()
}

pub fn evaluate(s: &str, values: &mut Values) -> String {
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
        return match find_var(s.trim(), values) {
            VarTypes::I(x) => x.to_string(),
            VarTypes::S(x) => x,
            VarTypes::N => String::new(),
        };
    }
    let temp = String::from(vals.pop_front().unwrap().trim());
    let mut result = match find_var(&temp, values) {
        VarTypes::N => VarTypes::S(temp),
        x => x,
    };
    while vals.len() > 0 {
        let operant = vals.pop_front().unwrap().chars().next().unwrap();
        let temp = String::from(vals.pop_front().unwrap().trim());
        let var = match find_var(&temp, values) {
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

pub fn compare(s: &str, values: &mut Values) -> char {
    use StateCompare::*;
    let mut vals: Vec<VarTypes> = Vec::new();
    let mut val = String::new();
    let mut state = Normal;
    let mut comparision: Comparision = equal;
    for c in s.chars() {
        match state {
            Normal => {
                match c {
                    '!' => state = ExclamationMark,
                    '=' => state = Equal,
                    '>' => state = Greater,
                    '<' => state = Less,
                    _ => {
                        val.push(c);
                        continue;
                    }
                }
                vals.push(find_var(&val, values));
                val = String::new();
                continue;
            }
            ExclamationMark => {
                match c {
                    '=' => comparision = inequal,
                    _ => {
                        val.push('!');
                        val.push(c);
                    }
                }
                state = Normal;
            }
            Equal => {
                match c {
                    '=' => comparision = equal,
                    _ => {
                        val.push('=');
                        val.push(c);
                    }
                }
                state = Normal
            }
            Less => {
                match c {
                    '=' => comparision = lessequal,
                    _ => {
                        comparision = less;
                        val.push(c)
                    }
                }
                state = Normal;
            }
            Greater => {
                match c {
                    '=' => comparision = greaterequal,
                    _ => {
                        comparision = greater;
                        val.push(c)
                    }
                }
                state = Normal
            }
        }
    }
    vals.push(find_var(&val, values));
    if vals.len() == 0 {
        return '0';
    }
    if vals.len() == 1 {
        if vals[0].get_type() == 'I' {
            if vals[0].get_i() == 0 {
                return '0';
            }
        }
        return '1';
    }
    if comparision(vals[0].clone(), vals[1].clone()) {
        return '1';
    }
    return '0';
}
