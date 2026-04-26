#![allow(dead_code)]

use crate::device_tree_utils;
use crate::libraries::console_utils::*;
use crate::libraries::thread_utils::*;

pub extern "C" fn new() -> ! {
    loop {
        print_string("SHELL$ ");
        let input = get_input_string();
        print_character('\n');
        match input.as_str() {
            "exit" => {
                thread_exit();
            }
            "release" => {
                thread_release();
            }
            "time" => {
                let time = riscv::register::time::read();
                println!("{}", time);
            }
            "harts" => {
                let device_tree_binary_ptr = crate::DEVICE_TREE_PTR.get().unwrap();
                let device_tree = device_tree_utils::get_device_tree(*device_tree_binary_ptr);
                let mut cpus = device_tree.cpus();
                while let Some(cpu) = cpus.next() {
                    let cpu_id = cpu.ids().first();
                    let clock_speed = cpu.timebase_frequency();
                    println!("id: {}\t@{}", cpu_id, clock_speed);
                }
            }
            "example" => {
                let args = ExampleArguments{
                    count: 400
                };

                let t1 = thread_create::<ExampleArguments>(example as *const u8 as usize, 1024, &args);
                thread_wait(t1);
                println!("Complete");
            }
            _ => {}
        }
    }
}

struct ExampleArguments {
    count: usize,
}

fn example(args: &ExampleArguments) -> ! {
    println!("Counting to {}",args.count);
    for _ in 0..args.count {}
    thread_exit();
}
