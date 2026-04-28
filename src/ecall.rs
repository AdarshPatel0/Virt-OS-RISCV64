use crate::{context, hart, print::print, thread, timer_interrupt};

pub fn call(context: &mut context::Context) {
    context.sepc = context.sepc + 4;
    let code = context.a[7];
    match code {
        0 => {
            let hart_info = unsafe { &mut *hart::get_hart_info_pointer() };
            match hart_info.current_thread_id {
                Some(current_thread_id) => {
                    let success = thread::kill_thread(current_thread_id);
                    if success {
                        context.a[0] = 1;
                    } else {
                        context.a[0] = 0;
                    }
                }
                None => {
                    context.a[0] = 0;
                }
            }
        }
        1 => {
            thread::schedule(context);
            timer_interrupt::update_timer();
        }
        2 => {
            let arguments = unsafe { core::slice::from_raw_parts(context.a[2] as *const u8, context.a[3]) };
            let thread_id = thread::create_thread(context.a[0], false, context.a[1], arguments);
            context.a[0] = thread_id;
        }
        3 => {
            let success = thread::cleanup_thread(context.a[0]);
            if !success {
                context.sepc = context.sepc - 4;
                thread::schedule(context);
                timer_interrupt::update_timer();
            }
        }
        9 => {
            use sbi::system_reset::*;
            let _ =system_reset(ResetType::Shutdown, ResetReason::NoReason);
        }
        10 => unsafe {
            print!("{}", core::char::from_u32_unchecked(context.a[0] as u32));
        },
        11 => unsafe {
            let slice = &*core::ptr::slice_from_raw_parts(context.a[0] as *const u8, context.a[1]);
            let string = core::str::from_utf8_unchecked(slice);
            print!("{}", string);
        },
        30 => {
            if let Some(character) = sbi::legacy::console_getchar() {
                context.a[0] = character as usize;
                context.a[1] = 1;
            } else {
                context.a[1] = 0;
            }
        }
        _ => {
            return;
        }
    }
}
