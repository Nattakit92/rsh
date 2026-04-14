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

pub fn input(mut history: Vec<String>) -> String {
    let mut buffer_: Vec<Vec<char>> = vec![Vec::new()];
    let mut buffer: Vec<char> = Vec::new();
    let mut display: String;
    let mut pos_x = 0;
    let mut pos_y = 0;
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
                if pos_x > 0 {
                    pos_x -= 1;
                }
                print!("\x1b[{}G", pos_x + START);
                continue;
            }
            RIGHT => {
                if pos_x < buffer.len() {
                    pos_x += 1;
                }
                print!("\x1b[{}G", pos_x + START);
                continue;
            }
            UP => {
                if pos_y > 0{
                    pos_y -= 1;
                    buffer = buffer_[pos_y].clone();
                    pos_x = min(pos_x,buffer.len());
                    print!("\x1b[1A\x1b[{}G",pos_x + START);
                    continue;
                }
            }
            DOWN => {
                if pos_y < buffer_.len() - 1{
                    pos_y += 1;
                    buffer = buffer_[pos_y].clone();
                    pos_x = min(pos_x,buffer.len());
                    print!("\x1b[1B\x1b[{}G",pos_x + START);
                    continue;
                }
            }
            CTRLLEFT => {
                while pos_x > 0 {
                    pos_x -= 1;
                    if STOPPER.contains(&buffer[pos_x]) {
                        break;
                    }
                }
                print!("\x1b[{}G", pos_x + START);
                continue;
            }
            CTRLRIGHT => {
                while pos_x + 1 < buffer.len() {
                    pos_x += 1;
                    if STOPPER.contains(&buffer[pos_x]) {
                        break;
                    }
                }
                print!("\x1b[{}G", pos_x + START);
                continue;
            }
            BACKSPACE => {
                if pos_x > 0 {
                    pos_x -= 1;
                    buffer.remove(pos_x);
                }
            }
            DELETE => {
                if pos_x < buffer.len() {
                    buffer.remove(pos_x);
                }
            }
            CTRLBACKSPACE => {
                if pos_x > 0 {
                    pos_x -= 1;
                    buffer.remove(pos_x);
                }
                while pos_x > 0 {
                    pos_x -= 1;
                    if STOPPER.contains(&buffer[pos_x]) {
                        break;
                    }
                    buffer.remove(pos_x);
                }
                if buffer.len() > 0 {
                    pos_x += 1;
                }
            }
            CTRLDELETE => {
                if pos_x < buffer.len() {
                    buffer.remove(pos_x);
                }
                while pos_x < buffer.len() {
                    if STOPPER.contains(&buffer[pos_x]) {
                        break;
                    }
                    buffer.remove(pos_x);
                }
            }
            NEWLINE => {
                break;
            }
            CTRLC => {}
            SHIFTENTER => {
                println!();
                pos_y += 1;
                buffer_.insert(pos_y, Vec::new());
                buffer = Vec::new();
                pos_x = 0;
            }
            _ => {
                let temp = str::from_utf8(&buf).expect("invalid utf8");
                let cs = temp.chars();
                for c in cs {
                    if c.is_control() {
                        continue;
                    }
                    buffer.insert(pos_x, c);
                    pos_x += 1;
                }
            }
        }
        buffer_[pos_y] = buffer.clone();
        display = buffer.iter().collect();
        print!("\r\x1b[2C\x1b[0K{}", display);
        print!("\x1b[{}G", pos_x + START);
    }
    println!("\x1b[{}B", buffer_.len() - pos_y - 1);
    raw_switch();
    buffer_
        .iter()
        .map(|line| {
            let mut s: String = line.iter().collect();
            s.push('\n');
            s
        })
        .collect::<Vec<_>>().join("\n")
}
