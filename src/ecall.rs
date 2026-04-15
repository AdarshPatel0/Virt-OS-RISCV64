use core::slice;

use crate::{println, thread};

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
        10 => {
            let string_pointer = context.a0 as *const u8;
            let string_length = context.a1;

            let slice = unsafe { slice::from_raw_parts(string_pointer, string_length) };
            match str::from_utf8(slice) {
                Ok(string) => {
                    println!("{}", string);
                }
                Err(_) => context.a0 = 1,
            }
        }
        11 => {
            println!("{}", context.a0);
        }
        _ => {
            return;
        }
    }
}
