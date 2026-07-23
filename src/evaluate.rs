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
    match s.parse::<f32>() {
        Ok(x) => {
            return F(x);
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
    match (var1,var2) {
        (I(a),I(b)) => I(a + b),
        (I(a),F(b)) => F(a as f32 + b),
        (F(a),F(b)) => F(a + b),
        (F(a),I(b)) => F(a + b as f32),
        (a,b) => S(a.get_s() + &b.get_s())
    }
}

fn sub(var1: VarTypes, var2: VarTypes) -> VarTypes {
    match (var1,var2) {
        (I(a),I(b)) => I(a - b),
        (I(a),F(b)) => F(a as f32 - b),
        (F(a),F(b)) => F(a - b),
        (F(a),I(b)) => F(a - b as f32),
        (_,_) => N
    }
}

fn divi(var1: VarTypes, var2: VarTypes) -> VarTypes {
    match var2 {
        F(0.0) | I(0) => return N,
        _ => ()
    }
    match (var1,var2) {
        (I(a),I(b)) => I(a / b),
        (I(a),F(b)) => F(a as f32 / b),
        (F(a),F(b)) => F(a / b),
        (F(a),I(b)) => F(a / b as f32),
        (_,_) => N
    }
}

fn mult(var1: VarTypes, var2: VarTypes) -> VarTypes {
    match (var1,var2) {
        (I(a),I(b)) => I(a * b),
        (I(a),F(b)) => F(a as f32 * b),
        (F(a),F(b)) => F(a * b),
        (F(a),I(b)) => F(a * b as f32),
        (_,_) => N
    }
}

fn pow(var1: VarTypes, var2: VarTypes) -> VarTypes {
    match (var1,var2) {
        (I(a),I(b)) => I((a as f32).powi(b) as i32),
        (I(a),F(b)) => F((a as f32).powf(b)),
        (F(a),F(b)) => F(a.powf(b)),
        (F(a),I(b)) => F(a.powf(b as f32)),
        (_,_) => N
    }
}

fn equal(var1: VarTypes, var2: VarTypes) -> bool {
    if var1 == var2 {
        return true;
    }
    false
}

fn isnum(var1: VarTypes, var2: VarTypes) -> bool {
    let mut b = true;
    match var1 {
        I(_) | F(_) => (),
        _ => b = false
    }
    match var2 {
        I(_) | F(_) => (),
        _ => b = false
    }
    b
}

fn inequal(var1: VarTypes, var2: VarTypes) -> bool {
    !equal(var1, var2)
}

fn greater(var1: VarTypes, var2: VarTypes) -> bool {
    match (var1,var2) {
        (I(a),I(b)) => a > b,
        (I(a),F(b)) => a as f32 > b,
        (F(a),F(b)) => a > b,
        (F(a),I(b)) => a > b as f32,
        (_,_) => false
    }
}

fn greaterequal(var1: VarTypes, var2: VarTypes) -> bool {
    if !isnum(var1.clone(), var2.clone()) {
        return false;
    }
    if equal(var1.clone(), var2.clone()) {
        return true;
    }
    greater(var1, var2)
}

fn less(var1: VarTypes, var2: VarTypes) -> bool {
    if !isnum(var1.clone(), var2.clone()) {
        return false;
    }
    !greaterequal(var1, var2)
}

fn lessequal(var1: VarTypes, var2: VarTypes) -> bool {
    if !isnum(var1.clone(), var2.clone()) {
        return false;
    }
    !greater(var1, var2)
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
            F(x) => x.to_string(),
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
        F(x) => x.to_string(),
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
