use crate::libraries::console_utils::println;
use crate::libraries::thread_utils::*;

pub extern "C" fn counter(args: &[u8]) -> ! {
    println!("{:?}", args);
    thread_exit();
}
