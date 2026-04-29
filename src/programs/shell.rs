#![allow(dead_code)]

use spin::Mutex;

use crate::device_tree_utils;
use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::libraries::thread_utils::*;

static SEM: Mutex<usize> = Mutex::new(0);

pub extern "C" fn new() -> ! {
    loop {
        print!("SHELL$ ");
        let input = get_input_string();
        println!();
        match input.as_str() {
            "exit" => {
                system_shutdown("System shutting down.");
            }
            "time" => {
                let time = riscv::register::time::read();
                println!("{}", time);
            }
            "sysinfo" => {
                let device_tree_binary_ptr = crate::DEVICE_TREE_PTR.get().unwrap();
                let device_tree = device_tree_utils::get_device_tree(*device_tree_binary_ptr);
                for node in device_tree.all_nodes() {
                    print!("{}: ", node.name);
                    if let Some(compatable) = node.compatible() {
                        for name in compatable.all() {
                            print!("{}", name);
                        }
                    }
                    println!();
                }
            }
            "example" => {
                Semaphore::set(&SEM, 2);
                let args = ExampleArguments { count: 10_000_000 };
                let t1 = thread_create::<ExampleArguments>(example as *const u8 as usize, 4096, &args);
                let t2 = thread_create::<ExampleArguments>(example as *const u8 as usize, 4096, &args);
                let t3 = thread_create::<ExampleArguments>(example as *const u8 as usize, 4096, &args);
                let t4 = thread_create::<ExampleArguments>(example as *const u8 as usize, 4096, &args);
                thread_wait(t1);
                thread_wait(t2);
                thread_wait(t3);
                thread_wait(t4);
            }
            _ => {}
        }
    }
}

struct ExampleArguments {
    count: usize,
}

fn example(args: &ExampleArguments) -> ! {
    Semaphore::wait(&SEM);
    println!("Starting");
    for _ in 0..args.count {}
    println!("Complete");
    Semaphore::post(&SEM);
    thread_exit();
}
