use core::str::FromStr;

use alloc::string::String;
use rsext4::Ext4FileSystem;
use rsext4::Jbd2Dev;
use virtio_drivers::transport::mmio::MmioTransport;

use crate::ext4_block_device::Ext4BlockDevice;
use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::virtio_hal::VirtIOHal;

pub fn new(block_dev: &mut Jbd2Dev<Ext4BlockDevice<VirtIOHal, MmioTransport>>) -> ! {
    let mut fs = Ext4FileSystem::mount(block_dev).unwrap();
    let s_volume_name = fs.superblock.s_volume_name;
    let volume_name = unsafe { core::str::from_utf8_unchecked(&s_volume_name) };
    let mut working_directory = String::from_str("/").unwrap();
    loop {
        print!("[{} {}]$ ", volume_name, working_directory);
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
                    fs.sync_filesystem(block_dev).unwrap();
                    fs.umount(block_dev).unwrap();
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
                "lsblk" => {
                    let block_devices = crate::devices::BLOCK_DEVICES.lock();
                    let block_device_ids = block_devices.iter();
                    for (block_device_id, block_device_mutex) in block_device_ids {
                        let block_device = block_device_mutex.lock();
                        let capacity = (block_device.capacity() * 512) as f64 / 1048576 as f64;
                        println!("Block Device id: {}\t Capacity: {}MB", block_device_id, capacity);
                    }
                }
                "ls" => {
                    let mut inode = rsext4::find_file(&mut fs, block_dev, &working_directory).unwrap();
                    let blocks = rsext4::loopfile::resolve_inode_block_allextend(&mut fs, block_dev, &mut inode).unwrap();
                    let mut block_data = alloc::vec::Vec::new();
                    for (_, block_num) in blocks {
                        let cache = fs.datablock_cache.get_or_load(block_dev, block_num).unwrap();
                        block_data.extend_from_slice(&cache.data);
                    }
                    let entries = rsext4::entries::classic_dir::list_entries(&block_data);
                    for entry in entries {
                        println!("{}", entry.name_str().unwrap_or_default());
                    }
                }
                "cd" => {
                    if let Some(path) = args.next() {
                        // Use find_file to look up an inode by path to verify it exists [1].
                        match rsext4::ext4::find_file(&mut fs, block_dev, path) {
                            Ok(_) => {
                                working_directory = String::from(path);
                            }
                            Err(_) => {
                                println!("Directory not found: {}", path);
                            }
                        }
                    }
                }
                "mkdir" => {
                    if let Some(path) = args.next() {
                        // Creates a directory and any missing parent directories [2].
                        rsext4::dir::mkdir(block_dev, &mut fs, path).unwrap();
                    }
                }
                "mkfile" => {
                    if let Some(path) = args.next() {
                        // Create a file entry, creating missing parent directories on demand [3].
                        rsext4::file::mkfile(block_dev, &mut fs, path, None, None).unwrap();
                    }
                }
                "rmdir" => {
                    if let Some(path) = args.next() {
                        // Remove a directory tree [4].
                        rsext4::file::delete_dir(&mut fs, block_dev, path).unwrap();
                    }
                }
                "rmfile" => {
                    if let Some(path) = args.next() {
                        // Remove a non-directory inode from its parent directory [5].
                        rsext4::file::delete_file(&mut fs, block_dev, path).unwrap();
                    }
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }
}
