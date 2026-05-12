#![allow(unused)]

use crate::{devices, libraries::thread_utils::thread_yield};

extern crate alloc;

pub fn get_ascii_char() -> u8 {
    loop {
        if let Some(console) = &mut *devices::CONSOLE.lock() {
            if let Ok(result) = console.recv(true) {
                if let Some(input) = result {
                    return input;
                }
            }
        }
    }
    return 0;
}

pub fn get_input_string() -> alloc::string::String {
    let mut input = alloc::string::String::new();
    let mut cursor_position = 0;
    loop {
        match get_ascii_char() {
            b'\x1b' => {
                if get_ascii_char() != b'[' {
                    continue;
                }
                match get_ascii_char() {
                    b'C' => {
                        if cursor_position < input.chars().count() {
                            cursor_position = cursor_position + 1;
                            print!("\x1b[C");
                        }
                    }
                    b'D' => {
                        if cursor_position > 0 {
                            cursor_position = cursor_position - 1;
                            print!("\x1b[D");
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
            b'\x7f' => {
                if cursor_position > 0 {
                    cursor_position = cursor_position - 1;
                    input.remove(cursor_position);
                    print!("\x1b[D");
                    let mut characters = input.chars().skip(cursor_position);
                    let mut i = 1;
                    while let Some(character) = characters.next() {
                        print!("{}", character);
                        i += 1;
                    }
                    print!(" ");
                    for _ in 0..i {
                        print!("\x1b[D");
                    }
                }
            }
            b'\r' => {
                return input;
            }
            graphic_ascii_character @ b'\x20'..=b'\x7e' => {
                input.insert(cursor_position, graphic_ascii_character as char);
                cursor_position = cursor_position + 1;
                print!("{}", graphic_ascii_character as char);
                let mut characters = input.chars().skip(cursor_position);
                let mut i = 0;
                while let Some(character) = characters.next() {
                    print!("{}", character);
                    i += 1;
                }
                for _ in 0..i {
                    print!("\x1b[D");
                }
            }
            _ => continue,
        }
    }
}

pub fn clear_screen() {
    print!("\x1b[2J");
    print!("\x1b[H");
}

use core::fmt::{self, Write};
struct Writer;

impl Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        if let Some(console) = &mut *devices::CONSOLE.lock() {
            console.send_bytes(string.as_bytes());
        }
        Ok(())
    }
}

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    Writer.write_fmt(args).unwrap();
}

macro_rules! print {
    ($($arg:tt)*) => {
        $crate::libraries::console_utils::print_args(format_args!($($arg)*));
    };
}

macro_rules! println {
    () => {
        $crate::libraries::console_utils::print!("\n");
    };
    ($fmt:expr) => {
        $crate::libraries::console_utils::print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::libraries::console_utils::print!(
            concat!($fmt, "\n"),
            $($arg)*
        );
    };
}

pub(crate) use print;
pub(crate) use println;
