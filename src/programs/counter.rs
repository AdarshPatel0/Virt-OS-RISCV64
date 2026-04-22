use crate::libraries::console_utils::println;
use crate::libraries::thread_utils::*;
#[repr(C)]
pub struct CounterArgs {
    pub n: usize,
    pub d: usize,
}

pub extern "C" fn new(args: &CounterArgs) -> ! {
    for i in 0..args.n {
        println!("{}", i);
        let a = riscv::register::time::read();
        loop {
            let b = riscv::register::time::read();
            if (b - a) > args.d {
                break;
            }
            thread_release();
        }
    }
    thread_exit();
}
