use crate::devices::RTC;
use crate::ext4_block_device::Ext4BlockDevice;
use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::libraries::thread_utils::Semaphore;
use crate::libraries::thread_utils::thread_create;
use crate::libraries::thread_utils::thread_exit;
use crate::libraries::thread_utils::thread_wait;
use crate::virtio_hal::VirtIOHal;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rsext4::Errno;
use rsext4::Ext4FileSystem;
use rsext4::Jbd2Dev;
use rsext4::disknode::Ext4Inode;
use shell_words::split;
use spin::Mutex;
use time::OffsetDateTime;
use time::UtcDateTime;

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
        let arguments = match split(&input) {
            Ok(arguments) => arguments,
            Err(error) => {
                println!("shell: {}", error);
                continue;
            }
        };
        let mut args = arguments.iter().peekable();

        if let Some(command) = args.next() {
            match command.as_str() {
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
                "date" => {
                    if let Some(rtc) = RTC.get() {
                        let timestamp = rtc.get_unix_timestamp();
                        let time = match UtcDateTime::from_unix_timestamp(timestamp as i64) {
                            Ok(date) => {
                                let (year, month, day) = date.to_calendar_date();
                                let weekday = date.weekday();
                                let (mut hour, minute, second) = date.as_hms();
                                let meridiem = match hour / 12 {
                                    0 => {
                                        format!("AM")
                                    }
                                    1 => {
                                        format!("PM")
                                    }
                                    _ => {
                                        format!("")
                                    }
                                };
                                hour = hour % 12;
                                format!("{} {} {} {:02}:{:02}:{:02} {} UTC {}", weekday, month, day, hour, minute, second, meridiem, year)
                            }
                            Err(error) => {
                                format!("date: {}", error)
                            }
                        };
                        println!("{}", time);
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
                            }
                        }
                    } else {
                        let path = resolve_path(&working_directory, "");
                        if let Err(error) = print_directory_contents(&mut fs, block_dev, &path) {
                            println!("{}: {}", command, error);
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
                "read" => {
                    if args.peek().is_some() {
                        for arg in args {
                            let path = resolve_path(&working_directory, &arg);
                            match rsext4::read_file(block_dev, &mut fs, &path) {
                                Ok(data) => {
                                    for value in data {
                                        print!("{}", value as char);
                                    }
                                }
                                Err(error) => {
                                    println!("{}: {}", command, error);
                                }
                            }
                        }
                    } else {
                        println!("Usage: {} <file_1 file_2 ...> ", command);
                    }
                }
                "write" => match args.next() {
                    Some(input_path) => {
                        let mut data = Vec::new();
                        let path = resolve_path(&working_directory, input_path);
                        for arg in args {
                            for value in arg.bytes() {
                                data.push(value);
                            }
                            data.push(b'\n');
                        }
                        if let Err(error) = rsext4::write_file(block_dev, &mut fs, &path, 0, &data) {
                            println!("{}: {}", command, error);
                        }
                    }
                    None => {
                        println!("Usage: {} <file> <data>", command);
                    }
                },
                "example1" => {
                    let t1 = thread_create::<()>(count as *const u8 as usize, 4096, &());
                    let t2 = thread_create::<()>(count as *const u8 as usize, 4096, &());
                    let t3 = thread_create::<()>(count as *const u8 as usize, 4096, &());
                    let t4 = thread_create::<()>(count as *const u8 as usize, 4096, &());
                    thread_wait(t1);
                    thread_wait(t2);
                    thread_wait(t3);
                    thread_wait(t4);
                }
                _ => {
                    println!("{}: command not found", command);
                }
            }
        }
    }
}

static SEM: Mutex<usize> = Mutex::new(2);

fn count() {
    Semaphore::wait(&SEM);
    println!("Counting...");
    for _ in 0..10000000 {}
    println!("Done");
    Semaphore::post(&SEM);
    thread_exit()
}

fn print_directory_contents(fs: &mut Ext4FileSystem, block_dev: &mut Jbd2Dev<Ext4BlockDevice<VirtIOHal, MmioTransport>>, path: &str) -> Result<(), String> {
    let mut inode = match find_inode(fs, block_dev, &path) {
        Ok(inode) => inode,
        Err(error) => return Err(format!("{}", error)),
    };
    if !inode.is_dir() {
        return Err(format!("{}: Not a directory", path));
    }
    let block_data = match get_inode_block_data(fs, block_dev, &mut inode) {
        Ok(data) => data,
        Err(error) => return Err(format!("{}", error)),
    };
    let entries = rsext4::entries::classic_dir::list_entries(&block_data);
    for entry in entries {
        let entry_name = match entry.file_type {
            2 => format!("\x1b[34m{}\x1b[0m", entry.name_str().unwrap_or_default()),
            _ => format!("{}", entry.name_str().unwrap_or_default()),
        };
        let permissions = {
            let flags = [0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001];

            let chars = ['r', 'w', 'x'];

            let mut result = String::with_capacity(9);

            for (i, flag) in flags.iter().enumerate() {
                if inode.permissions() & flag != 0 {
                    result.push(chars[i % 3]);
                } else {
                    result.push('-');
                }
            }

            result
        };
        let time = match OffsetDateTime::from_unix_timestamp(inode.ctime_ts(Ext4Inode::LARGE_INODE_SIZE).sec) {
            Ok(time) => {
                let (_, month, day) = time.to_calendar_date();
                let (hour, minute, _) = time.to_hms();
                format!("{} {} {:02}:{:02}", month, day, hour, minute)
            }
            Err(_) => {
                format!("Time/Date Error")
            }
        };
        println!("{} {} {} {} {}", permissions, inode.uid(), inode.gid(), time, entry_name);
    }
    Ok(())
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
