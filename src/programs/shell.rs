use fatfs::FileSystem;
use fatfs::LossyOemCpConverter;
use fatfs::NullTimeProvider;
use virtio_drivers::transport::mmio::MmioTransport;

use crate::libraries::console_utils::*;
use crate::libraries::system_utils::*;
use crate::virtio_hal::VirtIOHal;

pub type FatFileSystem = FileSystem<crate::virtio_fatfs::FatFsBlockDevice<VirtIOHal, MmioTransport<'static>>, NullTimeProvider, LossyOemCpConverter>;


pub extern "C" fn new(filesystem: FatFileSystem) -> ! {
    loop {
        print!("[{}]$ ", filesystem.volume_label());
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
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
    }
}
