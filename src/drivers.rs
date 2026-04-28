use alloc::vec::Vec;
use core::ptr::NonNull;
use fdt::Fdt;
use spin::Mutex;
use virtio_drivers::{
    device::blk::VirtIOBlk,
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use crate::virtio_hal::{self, VirtIOHal};

pub static BLOCK_DEVICES: Mutex<Vec<Mutex<VirtIOBlk<VirtIOHal, MmioTransport<'static>>>>> = Mutex::new(Vec::new());

pub fn load_drivers(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
        let reg = node.reg().unwrap().next().unwrap();
        let mmio_device_address = reg.starting_address as usize;
        let mmio_size = reg.size.unwrap();

        let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

        if let Ok(transport) = unsafe { MmioTransport::new(header, mmio_size) } {
            match transport.device_type() {
                DeviceType::Block => {
                    let mut block_devices = BLOCK_DEVICES.lock();
                    let block_device = VirtIOBlk::<virtio_hal::VirtIOHal, _>::new(transport).unwrap();
                    block_devices.push(Mutex::new(block_device));
                }
                DeviceType::Console => {}
                _ => {}
            }
        }
    }
}
