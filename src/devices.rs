use alloc::sync::Arc;
use core::ptr::NonNull;
use fdt::Fdt;
use riscv_goldfish::Rtc;
use spin::{Mutex, Once};
use virtio_drivers::{
    device::{blk::VirtIOBlk, console::VirtIOConsole},
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use crate::virtio_hal::VirtIOHal;

pub static CONSOLE: Mutex<Option<VirtIOConsole<VirtIOHal, MmioTransport>>> = Mutex::new(None);
pub static BLOCK_DEVICES: Mutex<slab::Slab<Arc<Mutex<VirtIOBlk<VirtIOHal, MmioTransport>>>>> = Mutex::new(slab::Slab::new());
pub static RTC: Once<Rtc> = Once::new();

pub fn load_virtio_devices(device_tree: &Fdt) {
    for node in device_tree.all_nodes() {
        if node.name.starts_with("virtio_mmio") {
            let reg = node.reg().unwrap().next().unwrap();
            let mmio_device_address = reg.starting_address as usize;
            let mmio_size = reg.size.unwrap();

            let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

            if let Ok(transport) = unsafe { MmioTransport::new(header, mmio_size) } {
                match transport.device_type() {
                    DeviceType::Block => {
                        let block_device = VirtIOBlk::<VirtIOHal, _>::new(transport).unwrap();
                        {
                            let mut block_devices = BLOCK_DEVICES.lock();
                            block_devices.insert(Arc::new(Mutex::new(block_device)));
                            drop(block_devices);
                        }
                    }
                    DeviceType::Console => {
                        let console = VirtIOConsole::<VirtIOHal, _>::new(transport).unwrap();
                        let mut global_console = CONSOLE.lock();
                        if global_console.is_none() {
                            *global_console = Some(console);
                        } else {
                            panic!("Multiple console devices detected.");
                        }
                    }
                    _ => {}
                }
            }
        } else if node.name.starts_with("rtc") {
            RTC.call_once(|| Rtc::new(node.reg().unwrap().next().unwrap().starting_address as usize));
        }
    }
}
