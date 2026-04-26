use core::ptr::NonNull;
use fdt::Fdt;
use spin::Mutex;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

use crate::virtio_hal;

pub static DISKS: spin::Once<slab::Slab<spin::Mutex<VirtIOBlk<virtio_hal::VirtIOHal, MmioTransport<'_>>>>> = spin::Once::new();

pub fn load_disks(device_tree: Fdt) {
    DISKS.call_once(|| {
        let mut slab: slab::Slab<spin::mutex::Mutex<VirtIOBlk<virtio_hal::VirtIOHal, MmioTransport<'_>>>> = slab::Slab::new();
        for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
            let reg = node.reg().unwrap().next().unwrap();
            let mmio_device_address = reg.starting_address as usize;
            let mmio_size = reg.size.unwrap();

            let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

            match unsafe { MmioTransport::new(header, mmio_size) } {
                Ok(transport) => {
                    if transport.device_type() == DeviceType::Block {
                        let disk: VirtIOBlk<virtio_hal::VirtIOHal, MmioTransport<'_>> = VirtIOBlk::<virtio_hal::VirtIOHal, _>::new(transport).unwrap();
                        slab.insert(Mutex::new(disk));
                    }
                }
                Err(_) => {}
            }
        }
        slab
    });
}
