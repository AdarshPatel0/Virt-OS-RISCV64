use crate::print::print;
use crate::{thread, timer_interrupt};

pub fn call(context: &mut thread::context::Context) {
    context.sepc = context.sepc + 4;
    let code = context.a7;
    match code {
        0 => {
            if let Some(current_thread) = thread::get_current_thread() {
                thread::delete_thread(current_thread);
                thread::schedule(context);
                timer_interrupt::update_timer();
            }
        }
        1 => {
            thread::schedule(context);
            timer_interrupt::update_timer();
        }
        2 => {
            context.a0 = thread::create_thread(context.a0, false, &[]);
        }
        3 => {
            let threads = thread::THREADS.lock();
            if let Some(thread) = threads.get(context.a0) {
                if thread.dead {
                    context.a0 = 0;
                    return;
                }
                context.a0 = 1;
                return;
            } else {
                context.a0 = 0;
                return;
            }
        }
        10 => unsafe {
            print!("{}", core::char::from_u32_unchecked(context.a0 as u32));
        },
        11 => unsafe {
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
