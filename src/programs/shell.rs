use crate::ext4_block_device::Ext4BlockDevice;
use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::virtio_hal::VirtIOHal;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rsext4::Errno;
use rsext4::Ext4FileSystem;
use rsext4::Jbd2Dev;
use rsext4::disknode::Ext4Inode;
use virtio_drivers::transport::mmio::MmioTransport;

pub fn new(block_dev: &mut Jbd2Dev<Ext4BlockDevice<VirtIOHal, MmioTransport>>) -> ! {
    let mut fs = Ext4FileSystem::mount(block_dev).unwrap();
    let s_volume_name = fs.superblock.s_volume_name;
    let volume_name = unsafe { core::str::from_utf8_unchecked(&s_volume_name) };
    let mut working_directory = "/".to_string();
    loop {
        print!("[{} {}]$ ", volume_name, working_directory);
        let input = get_input_string();
        println!();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut args = trimmed.split_whitespace().peekable();
        if let Some(command) = args.next() {
            match command {
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
                "cd" => match args.next() {
                    Some(input_path) => {
                        if args.next().is_some() {
                            println!("{}: Too many arguments", command);
                            continue;
                        }
                        let new_path = resolve_path(working_directory.as_str(), input_path);
                        match find_inode(&mut fs, block_dev, &new_path) {
                            Ok(inode) => {
                                if inode.is_dir() {
                                    working_directory = new_path;
                                } else {
                                    println!("{}: {}: Not a directory", command, new_path);
                                }
                            }
                            Err(error) => {
                                println!("{}: {}", command, error);
                            }
                        }
                    }
                    None => {
                        working_directory = "/".to_string();
                    }
                },
                "ls" => {
                    if args.peek().is_some() {
                        for arg in args {
                            let path = resolve_path(&working_directory, &arg);
                            if let Err(error) = print_directory_contents(&mut fs, block_dev, &path) {
                                println!("{}: {}", command, error);
                            } else {
                                println!();
                            }
                        }
                    } else {
                        let path = resolve_path(&working_directory, "");
                        if let Err(error) = print_directory_contents(&mut fs, block_dev, &path) {
                            println!("{}: {}", command, error);
                        } else {
                            println!();
                        }
                    }
                }
                "mkdir" => {
                    if args.peek().is_some() {
                        for arg in args {
                            let path = resolve_path(&working_directory, &arg);
                            if let Err(error) = rsext4::mkdir(block_dev, &mut fs, &path) {
                                println!("{}: {}", command, error);
                            }
                        }
                    } else {
                        println!("Usage: {} <directory_1 directory_2 ...> ", command);
                    }
                }
                "rmdir" => {
                    if args.peek().is_some() {
                        for arg in args {
                            let path = resolve_path(&working_directory, &arg);
                            if let Err(error) = rsext4::delete_dir(&mut fs, block_dev, &path) {
                                println!("{}: {}", command, error);
                            }
                        }
                    } else {
                        println!("Usage: {} <directory_1 directory_2 ...> ", command);
                    }
                }
                "mkfile" => {
                    if args.peek().is_some() {
                        for arg in args {
                            let path = resolve_path(&working_directory, &arg);
                            if let Err(error) = rsext4::mkfile(block_dev, &mut fs, &path, None, None) {
                                println!("{}: {}", command, error);
                            }
                        }
                    } else {
                        println!("Usage: {} <file_1 file_2 ...> ", command);
                    }
                }
                "rmfile" => {
                    if args.peek().is_some() {
                        for arg in args {
                            let path = resolve_path(&working_directory, &arg);
                            if let Err(error) = rsext4::delete_file(&mut fs, block_dev, &path) {
                                println!("{}: {}", command, error);
                            }
                        }
                    } else {
                        println!("Usage: {} <file_1 file_2 ...> ", command);
                    }
                }
                _ => {
                    println!("{}: command not found", command);
                }
            }
        }
    }
}

fn print_directory_contents(fs: &mut Ext4FileSystem, block_dev: &mut Jbd2Dev<Ext4BlockDevice<VirtIOHal, MmioTransport>>, path: &str) -> Result<(), String> {
    match find_inode(fs, block_dev, &path) {
        Ok(mut inode) => {
            if inode.is_dir() {
                match get_inode_block_data(fs, block_dev, &mut inode) {
                    Ok(block_data) => {
                        let entries = rsext4::entries::classic_dir::list_entries(&block_data);
                        for entry in entries {
                            match entry.file_type {
                                2 => {
                                    print!("\x1b[34m{}\x1b[0m ", entry.name_str().unwrap_or_default());
                                }
                                _ => {
                                    print!("{} ", entry.name_str().unwrap_or_default());
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(error) => Err(format!("{}", error)),
                }
            } else {
                Err(format!("{}: Not a directory", path))
            }
        }
        Err(error) => Err(format!("{}", error)),
    }
}

fn find_inode(fs: &mut Ext4FileSystem, block_dev: &mut Jbd2Dev<Ext4BlockDevice<VirtIOHal, MmioTransport>>, path: &str) -> Result<Ext4Inode, String> {
    match rsext4::find_file(fs, block_dev, &path) {
        Ok(inode) => Ok(inode),
        Err(error) => match error.code {
            Errno::ENOENT => Err(format!("{}: Not a file or directory", path)),
            _ => Err(format!("File system error: {:?}", error)),
        },
    }
}

fn get_inode_block_data(fs: &mut Ext4FileSystem, block_dev: &mut Jbd2Dev<Ext4BlockDevice<VirtIOHal, MmioTransport>>, inode: &mut rsext4::disknode::Ext4Inode) -> Result<Vec<u8>, String> {
    let blocks = match rsext4::loopfile::resolve_inode_block_allextend(fs, block_dev, inode) {
        Ok(blocks) => blocks,
        Err(error) => {
            return Err(format!("File system error: {:?}", error));
        }
    };
    let mut block_data = Vec::new();
    for (_, block_num) in blocks {
        let cache = match fs.datablock_cache.get_or_load(block_dev, block_num) {
            Ok(cached_block) => cached_block,
            Err(error) => {
                return Err(format!("File system error: {:?}", error));
            }
        };
        block_data.extend_from_slice(&cache.data);
    }
    return Ok(block_data);
}

fn resolve_path(working_directory: &str, input_path: &str) -> String {
    use alloc::vec::Vec;
    let new_path = if input_path.starts_with('/') {
        input_path.to_string()
    } else {
        let mut s = String::new();

        s.push_str(working_directory);

        if !working_directory.ends_with('/') {
            s.push('/');
        }

        s.push_str(input_path);
        s
    };

    let mut parts: Vec<&str> = Vec::new();

    for part in new_path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => {
                parts.push(part);
            }
        }
    }

    let mut result = String::from("/");
    result.push_str(&parts.join("/"));
    return result;
}
