use std::io::{Read, Write, stdin, stdout};
use termios::*;

const START: usize = 3;
const LEFT: u64 = 0x1b5b44;
const RIGHT: u64 = 0x1b5b43;
const BACKSPACE: u64 = 0x7f;
const DELETE: u64 = 0x1b5b337e;
const NEWLINE: u64 = 0x0a;
const CTRLLEFT: u64 = 0x1b5b313b3544;
const CTRLRIGHT: u64 = 0x1b5b313b3543;
const CTRLBACKSPACE: u64 = 0x8;
const CTRLDELETE: u64 = 0x1b5b333b357e;
const CTRLC: u64 = 0x3;
const STOPPER: &[char] = &[
    ' ', '\0', '.', '-', '\'', '\"', '\\', '/', '{', '(', '[', '}', ')', ']',
];

fn raw_switch() {
    let mut termios = Termios::from_fd(0).unwrap();
    termios.c_lflag ^= ICANON | ISIG | ECHO;

    _ = tcsetattr(0, TCSANOW, &termios);
}

pub fn input() -> String {
    let mut buffer: Vec<char> = Vec::new();
    let mut display: String = String::new();
    let mut pos = 0;
    raw_switch();
    print!("\x1b[5 q");
    loop {
        stdout().flush().expect("flush failed");
        let mut buf = [0u8; 8];
        let n = stdin().read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        let buf_ = u64::from_be_bytes(buf) >> ((8 - n) * 8);
        // println!("{:x}", buf);
        match buf_ {
            LEFT => {
                if pos > 0 {
                    pos -= 1;
                }
                print!("\x1b[{}G", pos + START);
                continue;
            }
            RIGHT => {
                if pos < buffer.len() {
                    pos += 1;
                }
                print!("\x1b[{}G", pos + START);
                continue;
            }
            CTRLLEFT => {
                while pos > 0 {
                    pos -= 1;
                    if STOPPER.contains(&buffer[pos]) {
                        break;
                    }
                }
                print!("\x1b[{}G", pos + START);
                continue;
            }
            CTRLRIGHT => {
                while pos + 1 < buffer.len() {
                    pos += 1;
                    if STOPPER.contains(&buffer[pos]) {
                        break;
                    }
                }
                print!("\x1b[{}G", pos + START);
                continue;
            }
            BACKSPACE => {
                if pos > 0 {
                    pos -= 1;
                    buffer.remove(pos);
                }
            }
            DELETE => {
                if pos < buffer.len() {
                    buffer.remove(pos);
                }
            }
            CTRLBACKSPACE => {
                if pos > 0 {
                    pos -= 1;
                    buffer.remove(pos);
                }
                while pos > 0 {
                    pos -= 1;
                    if STOPPER.contains(&buffer[pos]) {
                        break;
                    }
                    buffer.remove(pos);
                }
                if buffer.len() > 0 {
                    pos += 1;
                }
            }
            CTRLDELETE => {
                if pos < buffer.len() {
                    buffer.remove(pos);
                }
                while pos < buffer.len() {
                    if STOPPER.contains(&buffer[pos]) {
                        break;
                    }
                    buffer.remove(pos);
                }
            }
            NEWLINE => {
                break;
            }
            CTRLC => {}
            _ => {
                let temp = str::from_utf8(&buf).expect("invalid utf8");
                let cs = temp.chars();
                for c in cs {
                    if c.is_control() {
                        continue;
                    }
                    buffer.insert(pos, c);
                    pos += 1;
                }
            }
        }
        display = buffer.iter().collect();
        print!("\r\x1b[2C\x1b[0K{}", display);
        print!("\x1b[{}G", pos + START);
    }
    raw_switch();
    println!();
    display
}
