use crate::device_tree_utils;
use crate::libraries::console_utils::*;
use crate::libraries::thread_utils::*;

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
                println!("{}", time);
            }
            "info" => {
                let device_tree_binary_ptr = crate::DEVICE_TREE_PTR.get().unwrap();
                let device_tree = device_tree_utils::get_device_tree(*device_tree_binary_ptr);
                println!("{}",device_tree.root().model());
                let mut cpus = device_tree.cpus();
                println!("CPU's: ");
                while let Some(cpu) = cpus.next() {
                    let cpu_id = cpu.ids().first();
                    let clock_speed = cpu.timebase_frequency();
                    println!("id: {}\t@{}", cpu_id, clock_speed);
                }
            }
            "id" => {
                println!("{}", get_hart_id());
            }
            _ => {}
        }
    }
}
