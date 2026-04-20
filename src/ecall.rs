use crate::{print, thread};

pub fn call(context: &mut thread::context::Context) {
    context.sepc = context.sepc + 4;
    let code = context.a7;
    match code {
        0 => {
            if let Some(current_thread) = thread::get_current_thread() {
                thread::delete_thread(current_thread);
            }
        }
        10 => unsafe {
            print!("{}", core::char::from_u32_unchecked(context.a0 as u32));
        },
        11 => {
            print!("{}", context.a0);
        },
        12 => {
            print!("{}", context.a0 as isize);
        },
        20 => unsafe {
            let slice = &*core::ptr::slice_from_raw_parts(context.a0 as *const u8, context.a1);
            let string = core::str::from_utf8_unchecked(slice);
            print!("{}", string);
        },
        30 => {
            if let Some(input) = sbi::legacy::console_getchar() {
                context.a0 = input as usize;
                context.a1 = 1;
            } else {
                context.a1 = 0;
            }
        }
        _ => {
            return;
        }
    }
}