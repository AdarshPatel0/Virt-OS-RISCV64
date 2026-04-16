use crate::{print, println};

extern crate alloc;

fn get_ascii_char() -> u8 {
    loop {
        if let Some(char_byte) = sbi::legacy::console_getchar() {
            return char_byte;
        }
    }
}

fn get_input_string() -> alloc::string::String {
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
            127 => {
                if cursor_position > 0 {
                    cursor_position = cursor_position - 1;
                    input.remove(cursor_position);
                }
            }
            13 => {
                return input;
            }
            graphic_ascii_character @ 32..=126 => {
                input.insert(cursor_position, graphic_ascii_character as char);
                cursor_position = cursor_position + 1;
                print!("{}", graphic_ascii_character as char);
            }
            _ => continue,
        }
    }
}

pub fn shell() -> ! {
    loop {
        print!("SHELL$ ");
        println!("{}", get_input_string());
    }
}
