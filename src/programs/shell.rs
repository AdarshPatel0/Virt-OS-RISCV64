use crate::programs::console_utils::*;
use crate::programs::thread_utils::*;

pub fn shell() -> ! {
    loop {
        print_string("SHELL$ ");
        let input = get_input_string();
        print_character('\n');
        match input.as_str() {
            "exit" => {
                exit()
            },
            "time" => {
                let time = riscv::register::time::read();
                print_usize(time);
                print_character('\n');
            },
            _ => {}
        }
    }
}
