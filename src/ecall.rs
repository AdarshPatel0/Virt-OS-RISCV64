use core::slice;

use crate::{print, thread::Context};

pub fn call(context: &mut Context) {
    context.sepc = context.sepc + 4;
    let code = context.a7;
    match code {
        0 => {
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
        1 => {
            print!("{}", context.a0);
        }
        _ => {
            return;
        }
    }
}
