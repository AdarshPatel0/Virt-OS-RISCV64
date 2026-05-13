use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::libraries::thread_utils::*;

pub fn new() -> ! {
    let ref mut block_dev = {
        match crate::devices::BLOCK_DEVICES.lock().get_mut(0) {
            Some(virtio_block_device) => {
                let ext4_block_device = crate::ext4_block_device::Ext4BlockDevice::new(virtio_block_device.clone());
                rsext4::Jbd2Dev::initial_jbd2dev(0, ext4_block_device, true)
            }
            None => {
                println!("Error: No block device found. Terminating Shell");
                thread_exit();
            }
        }
    };
    let ref mut fs = match rsext4::Ext4FileSystem::mount(block_dev) {
        Ok(fs) => fs,
        Err(error) => {
            println!("{:?}", error);
            thread_exit();
        }
    };
    let root_inode = fs.get_root(block_dev).unwrap();
    let ref mut working_inode = root_inode.clone();

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
                    println!("Available commands: help, echo, clear, exit, disks");
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
                "lsdisk" => {
                    let block_devices = crate::devices::BLOCK_DEVICES.lock();
                    let block_device_ids = block_devices.iter();
                    for (block_device_id, block_device_mutex) in block_device_ids {
                        let block_device = block_device_mutex.lock();
                        let capacity = (block_device.capacity() * 512) as f64 / 1048576 as f64;
                        println!("Disk id: {}, Capacity: {}MB", block_device_id, capacity);
                    }
                }
                "ls" => {
                    let blocks = rsext4::loopfile::resolve_inode_block_allextend(fs, block_dev, working_inode).unwrap();
                    let ref mut block_data = alloc::vec::Vec::new();
                    for (_,block_num) in blocks {
                        let cached = fs.datablock_cache.get_or_load(block_dev, block_num).unwrap();
                        block_data.extend_from_slice(&cached.data);
                    }
                    let entries = rsext4::entries::classic_dir::list_entries(block_data);
                    for (i, entry) in entries.iter().enumerate() {
                        print!("{}", entry.name_str().unwrap_or_default());
                        if i < entries.len() - 1 {
                            print!(", ");
                        }
                    }
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }
}
