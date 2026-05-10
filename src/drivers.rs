use core::ptr::NonNull;
use fdt::Fdt;
use slab::Slab;
use spin::Mutex;
use virtio_drivers::{
    device::{blk::VirtIOBlk, console::VirtIOConsole},
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use crate::{virtio_block_wrapper::VirtioBlockWrapper, virtio_hal::VirtIOHal};

pub static CONSOLE: Mutex<Option<VirtIOConsole<VirtIOHal, MmioTransport>>> = Mutex::new(None);

pub fn load_drivers(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
        let reg = node.reg().unwrap().next().unwrap();
        let mmio_device_address = reg.starting_address as usize;
        let mmio_size = reg.size.unwrap();

        let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

        if let Ok(transport) = unsafe { MmioTransport::new(header, mmio_size) } {
            match transport.device_type() {
                DeviceType::Block => {
                    insert_block_device(transport);
                }
                DeviceType::Console => {
                    let console = VirtIOConsole::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
                    *CONSOLE.lock() = Some(console);
                }
                _ => {}
            }
        }
    }
}

fn insert_block_device(transport: MmioTransport<'static>) {
    let block_device = VirtIOBlk::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
    
    let mut journaling_device = rsext4::blockdev::Jbd2Dev::initial_jbd2dev(0, VirtioBlockWrapper::new(block_device), true);

    match rsext4::ext4::Ext4FileSystem::mount(&mut journaling_device) {
        Ok(filesystem) => {

        },
        Err(error) => {
            crate::print::println!("{}", error);
        },
    }
}
