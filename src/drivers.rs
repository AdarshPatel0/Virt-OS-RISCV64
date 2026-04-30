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

use crate::{print::println, virtio_hal::VirtIOHal};

pub static CONSOLE: Mutex<Option<VirtIOConsole<VirtIOHal, MmioTransport>>> = Mutex::new(None);
pub static BLOCK_DEVICES: Mutex<Slab<Mutex<VirtIOBlk<VirtIOHal, MmioTransport>>>> = Mutex::new(Slab::new());

pub fn load_drivers(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
        let reg = node.reg().unwrap().next().unwrap();
        let mmio_device_address = reg.starting_address as usize;
        let mmio_size = reg.size.unwrap();

        let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

        if let Ok(transport) = unsafe { MmioTransport::new(header, mmio_size) } {
            match transport.device_type() {
                DeviceType::Block => {
                    let id = transport.vendor_id();
                    insert_block_device(transport);
                    println!("Loaded Block Device: {}.", id);
                }
                DeviceType::Console => {
                    let console = VirtIOConsole::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
                    *CONSOLE.lock() = Some(console);
                    println!("Console loaded.");
                }
                _ => {}
            }
        }
    }
}

fn insert_block_device(transport: MmioTransport<'static>) {
    let block_device = VirtIOBlk::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
    BLOCK_DEVICES.lock().insert(Mutex::new(block_device));
}