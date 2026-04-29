use core::ptr::NonNull;
use fdt::Fdt;
use spin::Mutex;
use virtio_drivers::{
    device::{blk::VirtIOBlk, console::VirtIOConsole},
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use crate::virtio_hal::VirtIOHal;

pub static DISK: Mutex<Option<VirtIOBlk<VirtIOHal,MmioTransport>>> = Mutex::new(None);
pub static CONSOLE: Mutex<Option<VirtIOConsole<VirtIOHal,MmioTransport>>> = Mutex::new(None);

pub fn load_drivers(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
        let reg = node.reg().unwrap().next().unwrap();
        let mmio_device_address = reg.starting_address as usize;
        let mmio_size = reg.size.unwrap();

        let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

        if let Ok(transport) = unsafe { MmioTransport::new(header, mmio_size) } {
            match transport.device_type() {
                DeviceType::Block => {
                    let disk = VirtIOBlk::<VirtIOHal,MmioTransport<'_>>::new(transport).unwrap();
                    let mut disk_mutex = DISK.lock();
                    *disk_mutex = Some(disk);
                }
                DeviceType::Console => {
                    let console = VirtIOConsole::<VirtIOHal,MmioTransport<'_>>::new(transport).unwrap();
                    let mut console_mutex = CONSOLE.lock();
                    *console_mutex = Some(console);
                }
                _ => {}
            }
        }
    }
}
