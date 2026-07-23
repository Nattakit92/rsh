use crate::{Values, VarTypes};
use VarTypes::*;
use std::{collections::{HashMap, VecDeque}};

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
        return vars.get(s).unwrap().clone();
    }
    match s.parse::<i32>() {
        Ok(x) => {
            return I(x)
        },
        Err(_) => (),
    }
    N
}

fn add(var1: VarTypes, var2: VarTypes) -> VarTypes {
    if var1 == N && var2 == N {
        return N;
    }
    if var1 == N {
        return var2;
    }
    if var2 == N {
        return var1;
    }
    if let I(_) = var1 && let I(_) = var2 {
        return I(var1.get_i() + var2.get_i());
    }
    S(var1.get_s() + &var2.get_s())
}

fn sub(var1: VarTypes, var2: VarTypes) -> VarTypes {
    if let I(_) = var1 && let I(_) = var2 {
        return I(var1.get_i() - var2.get_i());
    }
    N
}

fn divi(var1: VarTypes, var2: VarTypes) -> VarTypes {
    if let I(_) = var1 && let I(_) = var2 {
        return I(var1.get_i() / var2.get_i());
    }
    N
}

fn mult(var1: VarTypes, var2: VarTypes) -> VarTypes {
    if let I(_) = var1 && let I(_) = var2 {
        return I(var1.get_i() * var2.get_i());
    }
    N
}

fn pow(var1: VarTypes, var2: VarTypes) -> VarTypes {
    if let I(_) = var1 && let I(_) = var2 {
        if var2.get_i() < 0 {
            return I(1 / var1.get_i().pow((-var2.get_i()) as u32));
        }
        return I(var1.get_i().pow(var2.get_i() as u32));
    }
    N
}

fn equal(var1: VarTypes, var2: VarTypes) -> bool {
    if var1 == var2 {
        return true;
    }
    false
}

fn isint(var1: VarTypes, var2: VarTypes) -> bool {
    if let I(_) = var1 && let I(_) = var2{
        return true;
    }
    false
}

fn inequal(var1: VarTypes, var2: VarTypes) -> bool {
    !equal(var1, var2)
}

fn greater(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(var1.clone(), var2.clone()) {
        return false;
    }
    return var1.get_i() > var2.get_i();
}

fn greaterequal(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(var1.clone(), var2.clone()) {
        return false;
    }
    if equal(var1.clone(), var2.clone()) {
        return true;
    }
    var1.get_i() > var2.get_i()
}

fn less(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(var1.clone(), var2.clone()) {
        return false;
    }
    return var1.get_i() < var2.get_i();
}

fn lessequal(var1: VarTypes, var2: VarTypes) -> bool {
    if !isint(var1.clone(), var2.clone()) {
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
            I(x) => x.to_string(),
            S(x) => x,
            B(x) => x.to_string(),
            N => String::new(),
        };
    }
    let temp = String::from(vals.pop_front().unwrap().trim());
    let mut result = match find_var(&temp, values) {
        N => S(temp),
        x => x,
    };
    while vals.len() > 0 {
        let operant = vals.pop_front().unwrap().chars().next().unwrap();
        let temp = String::from(vals.pop_front().unwrap().trim());
        let var = match find_var(&temp, values) {
            N => S(temp),
            x => x,
        };
        result = operations.get(&operant).unwrap()(result.clone(), var);
    }
    return match result {
        I(x) => x.to_string(),
        S(x) => x,
        B(x) => x.to_string(),
        N => String::new(),
    };
}

pub fn compare(s: &str, values: &mut Values) -> String {
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
        return String::from("false");
    }
    if vals.len() == 1 {
        if let I(_) = vals[0] {
            if vals[0].get_i() == 0 {
                return String::from("false");
            }
        }
        return String::from("true");
    }
    if comparision(vals[0].clone(), vals[1].clone()) {
        return String::from("true");
    }
    return String::from("false");
}
