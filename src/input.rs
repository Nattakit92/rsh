use std::io::{Read, Write, stdin, stdout};
use std::cmp::min;
use termios::*;

const START: usize = 3;
const LEFT: u128 = 0x1b5b44;
const RIGHT: u128 = 0x1b5b43;
const UP: u128 = 0x1b5b41;
const DOWN: u128 = 0x1b5b42;
const BACKSPACE: u128 = 0x7f;
const DELETE: u128 = 0x1b5b337e;
const NEWLINE: u128 = 0x0a;
const CTRLLEFT: u128 = 0x1b5b313b3544;
const CTRLRIGHT: u128 = 0x1b5b313b3543;
const CTRLBACKSPACE: u128 = 0x8;
const CTRLDELETE: u128 = 0x1b5b333b357e;
const CTRLC: u128 = 0x3;
const SHIFTENTER: u128 = 0x1b5b32373b323b31337e;
const STOPPER: &[char] = &[
    ' ', '\0', '.', '-', '\'', '\"', '\\', '/', '{', '(', '[', '}', ')', ']',
];

fn raw_switch() {
    let mut termios = Termios::from_fd(0).unwrap();
    termios.c_lflag ^= ICANON | ISIG | ECHO;

    _ = tcsetattr(0, TCSANOW, &termios);
}

#[derive(Clone)]
struct Buffer{
    data: Vec<String>,
    x: usize,
    y: usize,
}

impl Buffer{
    fn new() -> Self{
        Buffer { data: vec![String::from(String::new())], x: 0, y: 0 }
    }

    fn insert(&mut self, c: char){
        self.data[self.y].insert(self.x,c);
        self.x += 1;
    }

    fn x_inc(&mut self) -> bool{
        if self.x < self.data[self.y].len() {
            self.x += 1;
            return true;
        }
        false
    }

    fn x_dec(&mut self) -> bool{
        if self.x > 0 {
            self.x -= 1;
            return true;
        }
        false
    }

    fn y_inc(&mut self) -> bool{
        if self.y < self.data.len() - 1{
            self.y += 1;
            self.x = min(self.x, self.data[self.y].len());
            return true;
        }
        false
    }

    fn y_dec(&mut self) -> bool{
        if self.y > 0{
            self.y -= 1;
            self.x = min(self.x, self.data[self.y].len());
            return true;
        }
        false
    }

    fn get_char(&mut self) -> char{
        if self.x == 0{
            return '\0';
        }
        self.data[self.y].chars().nth(self.x-1).unwrap()
    }

    fn remove(&mut self) -> bool{
        if self.x_dec() {
            self.data[self.y].remove(self.x);
            return true;
        }
        false
    }

    fn get_line(&self) -> String{
        self.data[self.y].clone()
    }

    fn new_line(&mut self){
        self.data.insert(self.y + 1, String::new());
        self.y_inc();
    }

    fn del_line(&mut self){
        if self.y > 0{
            self.data.remove(self.y);
            self.y -= 1;
            self.x = self.data[self.y].len();
            print!("\x1b[1A\x1b[{}G", self.x + START);
        }
    }

    fn print(&self){
        for i in 0..self.y{
            println!("\x1b[{}G{}", START, self.data[i]);
        }
        print!("\x1b[{}G{}", START, self.data[self.y]);
    }
}

impl From<Buffer> for String{
    fn from(buf: Buffer) -> String{
        buf.data.join("\n")
    }
}

impl From<String> for Buffer{
    fn from(s: String) -> Buffer{
        let mut temp: Vec<String> = vec![String::new()];
        let mut i = 0;
        for c in s.chars(){
            if c == '\n'{
                temp.push(String::new());
                i+=1;
                continue;
            }
            temp[i].push(c);
        }
        Buffer { data: temp.clone(), y: temp.len()-1 , x: temp[temp.len()-1].len() }
    }
}

pub fn input(mut history: Vec<String>) -> String {
    let mut buffer = Buffer::new();
    let mut his_in = history.len();
    history.push(String::new());
    raw_switch();
    print!("\x1b[5 q");
    loop {
        stdout().flush().expect("flush failed");
        let mut buf = [0u8; 16];
        let n = stdin().read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        let buf_ = u128::from_be_bytes(buf) >> ((16 - n) * 8);
        match buf_ {
            LEFT => {
                buffer.x_dec();
                print!("\x1b[{}G", buffer.x + START);
                continue;
            }
            RIGHT => {
                buffer.x_inc();
                print!("\x1b[{}G", buffer.x + START);
                continue;
            }
            UP => {
                if buffer.y_dec(){
                    print!("\x1b[1A\x1b[{}G", buffer.x + START);
                    continue;
                }else{
                    if his_in > 0{
                        print!("\x1b[0J");
                        history[his_in] = String::from(buffer);
                        his_in -= 1;
                        buffer = Buffer::from(history[his_in].clone());
                        buffer.print();
                    }
                }
            }
            DOWN => {
                if buffer.y_inc(){
                    print!("\x1b[1B\x1b[{}G", buffer.x + START);
                    continue;
                }else{
                    if his_in < history.len() - 1{
                        while buffer.y_dec(){
                            print!("\x1b[1A");
                        }
                        print!("\x1b[0J");
                        history[his_in] = String::from(buffer);
                        his_in += 1;
                        buffer = Buffer::from(history[his_in].clone());
                        buffer.print();
                    }
                }
            }
            CTRLLEFT => {
                while buffer.x_dec() {
                    if STOPPER.contains(&buffer.get_char()) {
                        break;
                    }
                }
                print!("\x1b[{}G", buffer.x + START);
                continue;
            }
            CTRLRIGHT => {
                while buffer.x_inc() {
                    if STOPPER.contains(&buffer.get_char()) {
                        break;
                    }
                }
                print!("\x1b[{}G", buffer.x + START);
                continue;
            }
            BACKSPACE => {
                if !buffer.remove(){
                    buffer.del_line();
                }
            }
            DELETE => {
                if buffer.x_inc() {
                    buffer.remove();
                }
            }
            CTRLBACKSPACE => {
                buffer.remove();
                while buffer.remove() {
                    if STOPPER.contains(&buffer.get_char()) {
                        break;
                    }
                }
            }
            CTRLDELETE => {
                if buffer.x_inc() {
                    buffer.remove();
                }
                while buffer.x_inc() {
                    if STOPPER.contains(&buffer.get_char()) {
                        break;
                    }
                    buffer.remove();
                }
            }
            NEWLINE => {
                break;
            }
            CTRLC => {}
            SHIFTENTER => {
                println!();
                buffer.new_line();
            }
            _ => {
                let temp = str::from_utf8(&buf).expect("invalid utf8");
                let cs = temp.chars();
                for c in cs {
                    if c.is_control() {
                        continue;
                    }
                    buffer.insert(c);
                }
            }
        }

        print!("\r\x1b[{}C\x1b[0K{}", START - 1, buffer.get_line());
        print!("\x1b[{}G", buffer.x + START);
    }
    for _ in 0..buffer.data.len() - buffer.y{
        println!();
    }
    raw_switch();
    String::from(buffer)
}
