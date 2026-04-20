use crate::device_tree_utils;
use crate::programs::console_utils::*;
use crate::programs::thread_utils::*;

pub fn shell() -> ! {
    loop {
        print_string("SHELL$ ");
        let input = get_input_string();
        print_character('\n');
        match input.as_str() {
            "exit" => {
                exit();
            }
            "time" => {
                let time = riscv::register::time::read();
                print_usize(time);
                print_character('\n');
            }
            "info" => {
                let device_tree_binary_ptr = crate::DEVICE_TREE_PTR.get().unwrap();
                let device_tree = device_tree_utils::get_device_tree(*device_tree_binary_ptr);
                print_string(device_tree.root().model());
                print_character('\n');
                let mut cpus = device_tree.cpus();
                print_string("CPU's: ");
                print_character('\n');
                while let Some(cpu) = cpus.next() {
                    let cpu_id = cpu.ids().first();
                    let clock_speed = cpu.timebase_frequency();
                    print_string("id:");
                    print_usize(cpu_id);
                    print_character('@');
                    print_usize(clock_speed);
                    print_character('\n');
                }
            }
            _ => {}
        }
    }
}
