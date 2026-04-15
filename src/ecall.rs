use core::slice;

use crate::{print, thread, timer_interrupt};

pub fn call(context: &mut thread::context::Context) {
    context.sepc = context.sepc + 4;
    let code = context.a7;
    match code {
        0 => {
            if let Some(current_thread) = thread::get_current_thread() {
                thread::delete_thread(current_thread);
                thread::schedule(context);
            }
        }
        1 => {
            thread::schedule(context);
            timer_interrupt::update_timer();
        }
        10 => {
            let character = context.a0 as u8 as char;
            print!("{}", character);
        }
        11 => {
            let string_pointer = context.a0 as *const u8;
            let string_length = context.a1;

            let slice = unsafe { slice::from_raw_parts(string_pointer, string_length) };
            match str::from_utf8(slice) {
                Ok(string) => {
                    print!("{}", string);
                }
                Err(_) => context.a0 = 1,
            }
        }
        12 => {
            print!("{}", context.a0 as usize);
        }
        13 => {
            print!("{}", context.a0 as isize);
        }
        14 => loop {
            match sbi::legacy::console_getchar() {
                Some(character) => {
                    context.a0 = character as usize;
                    print!("{}", context.a0 as u8 as char);
                }
                None => {
                    continue;
                }
            }
        },
        _ => {
            return;
        }
    }
}

#[allow(dead_code)]
pub enum Ecall {
    Exit,
    Yield,
    PrintChar,
    PrintString,
    PrintUsize,
    PrintIsize,
    GetChar,
}

pub fn get_code(call: Ecall) -> usize {
    match call {
        Ecall::Exit => 0,
        Ecall::Yield => 1,
        Ecall::PrintChar => 10,
        Ecall::PrintString => 11,
        Ecall::PrintUsize => 12,
        Ecall::PrintIsize => 13,
        Ecall::GetChar => 14,
    }
}
