extern crate alloc;

pub fn get_ascii_char() -> u8 {
    let mut char: usize;
    let mut success: usize;
    loop {
        unsafe {
            core::arch::asm!(
                "
                li a7, 30
                ecall
                ",
                out("a0") char,
                out("a1") success
            );
        }
        if success != 0 {
            return char as u8;
        }
    }
}

pub fn print_character(character: char) {
    unsafe {
        core::arch::asm!(
            "
            li a7, 10
            ecall
            ",
            in("a0") character as usize
        )
    }
}

pub fn print_string(string: &str) {
    let string_address = string.as_ptr() as usize;
    let string_length = string.len();
    unsafe {
        core::arch::asm!(
            "
            li a7, 20
            ecall
            ",
            in("a0") string_address,
            in("a1") string_length
        )
    }
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
                            print_string("\x1b[C");
                        }
                    }
                    b'D' => {
                        if cursor_position > 0 {
                            cursor_position = cursor_position - 1;
                            print_string("\x1b[D");
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
                    print_string("\x1b[D");
                    let mut characters = input.chars().skip(cursor_position);
                    let mut i = 1;
                    while let Some(character) = characters.next() {
                        print_character(character);
                        i += 1;
                    }
                    print_string(" ");
                    for _ in 0..i {
                        print_string("\x1b[D");
                    }
                }
            }
            b'\r' => {
                return input;
            }
            graphic_ascii_character @ b'\x20'..=b'\x7e' => {
                input.insert(cursor_position, graphic_ascii_character as char);
                cursor_position = cursor_position + 1;
                print_character(graphic_ascii_character as char);
                let mut characters = input.chars().skip(cursor_position);
                let mut i = 0;
                while let Some(character) = characters.next() {
                    print_character(character);
                    i += 1;
                }
                for _ in 0..i {
                    print_string("\x1b[D");
                }
            }
            _ => continue,
        }
    }
}

use core::fmt::{self, Write};
struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print_string(s);
        Ok(())
    }
}

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    Console.write_fmt(args).unwrap();
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
