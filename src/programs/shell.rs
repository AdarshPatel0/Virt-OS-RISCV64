use crate::libraries::console_utils;

pub extern "C" fn new() -> ! {
    loop {
        console_utils::print!("SHELL$ ");
        let input = console_utils::get_input_string();
        console_utils::println!();
    }
}