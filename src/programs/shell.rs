use alloc::string::String;
use fatfs::FileSystem;
use fatfs::LossyOemCpConverter;
use fatfs::NullTimeProvider;
use virtio_drivers::transport::mmio::MmioTransport;

use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::virtio_hal::VirtIOHal;

pub type FatFileSystem = FileSystem<crate::virtio_fatfs::FatFsBlockDevice<VirtIOHal, MmioTransport<'static>>, NullTimeProvider, LossyOemCpConverter>;

pub fn new(filesystem: FatFileSystem) -> ! {
    let root_dir = filesystem.root_dir();
    let mut current_dir = filesystem.root_dir();
    let mut current_path = String::from("/");
    loop {
        print!("[{} {}]$ ", filesystem.volume_label(), current_path);
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
                    println!("Available commands: help, echo, clear, reboot");
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
                "disk" => {
                    if let Some(arg) = args.next() {
                        match arg {
                            "list" => {
                                let block_devices = crate::drivers::BLOCK_DEVICES.lock();
                                let block_device_ids = block_devices.iter();
                                for (block_device_id, block_device_mutex) in block_device_ids {
                                    let block_device = block_device_mutex.lock();
                                    let capacity = (block_device.capacity() * 512) as f64 / 1048576 as f64;
                                    println!("Device id: {}, Capacity: {}MB", block_device_id, capacity);
                                }
                            }
                            _ => {
                                println!("Error: Unknown argument: {}", arg);
                            }
                        }
                    } else {
                        println!("Error: No arguments provided");
                    }
                }
                "cd" => {
                    if let Some(path) = args.next() {
                        match path {
                            "." => {}
                            ".." => {
                                if current_path != "/" {
                                    if let Some(last_slash) = current_path.rfind('/') {
                                        if last_slash == 0 {
                                            current_path = String::from("/");
                                        } else {
                                            current_path.truncate(last_slash);
                                        }
                                        current_dir = root_dir.open_dir(&current_path).unwrap_or(filesystem.root_dir());
                                    }
                                }
                            }
                            "/" => {
                                current_dir = filesystem.root_dir();
                                current_path = String::from("/");
                            }
                            _ => match current_dir.open_dir(path) {
                                Ok(dir) => {
                                    current_dir = dir;
                                    if current_path != "/" {
                                        current_path.push('/');
                                    }
                                    current_path.push_str(path);
                                }
                                Err(_) => {
                                    println!("Error: Directory not found: {}", path);
                                }
                            },
                        }
                    } else {
                        println!("Usage: cd <path>");
                    }
                }
                "ls" => {
                    for entry in current_dir.iter() {
                        match entry {
                            Ok(e) => {
                                println!("{}", e.file_name());
                            }
                            Err(_) => {
                                println!("Error reading directory entry");
                            }
                        }
                    }
                }
                "mkdir" => {
                    if let Some(name) = args.next() {
                        if let Err(e) = current_dir.create_dir(name) {
                            println!("Error creating directory: {:?}", e);
                        }
                    } else {
                        println!("Usage: mkdir <name>");
                    }
                }
                "touch" => {
                    if let Some(name) = args.next() {
                        match current_dir.create_file(name) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error creating file: {:?}", e);
                            }
                        }
                    } else {
                        println!("Usage: touch <filename>");
                    }
                }
                "rm" | "rmdir" => {
                    if let Some(name) = args.next() {
                        if let Err(e) = current_dir.remove(name) {
                            println!("Error removing {}: {:?}", name, e);
                        }
                    } else {
                        println!("Usage: {} <name>", command);
                    }
                }
                "pwd" => {
                    println!("{}", current_path);
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }
}
