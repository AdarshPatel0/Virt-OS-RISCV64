use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;

pub fn new() -> ! {
    loop {
        print!("[SHELL]$ ");
        let input = get_input_string();
        println!();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut args = trimmed.split_whitespace();
        if let Some(command) = args.next() {
            match command {
                "help" => {
                    println!("Available commands: help, echo, clear, exit, disk, cd, ls, mkdir, mkfile, rm, pwd, read, write");
                }
                "exit" => {
                    system_shutdown("Shutdown command executed");
                }
                "clear" => {
                    clear_screen();
                }
                "echo" => {
                    for arg in args {
                        print!("{}", arg);
                        print!(" ");
                    }
                    println!();
                }
                "disks" => {
                    let block_devices = crate::devices::BLOCK_DEVICES.lock();
                    let block_device_ids = block_devices.iter();
                    for (block_device_id, block_device_mutex) in block_device_ids {
                        let block_device = block_device_mutex.lock();
                        let capacity = (block_device.capacity() * 512) as f64 / 1048576 as f64;
                        println!("Disk id: {}, Capacity: {}MB", block_device_id, capacity);
                    }
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }
}
