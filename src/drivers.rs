use core::ptr::NonNull;
use fdt::Fdt;
use riscv_plic::{PLICRegs, Plic};
use spin::Mutex;
use virtio_drivers::{
    device::{blk::VirtIOBlk, console::VirtIOConsole},
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use crate::{print::println, virtio_hal::VirtIOHal};

pub static DISK: Mutex<Option<VirtIOBlk<VirtIOHal, MmioTransport>>> = Mutex::new(None);
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
                    let disk = VirtIOBlk::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
                    let mut disk_mutex = DISK.lock();
                    *disk_mutex = Some(disk);
                    println!("Block device loaded.");
                }
                DeviceType::Console => {
                    for property in node.properties() {
                        if property.name == "interrupts" {
                            let value = u32::from_be(unsafe {*(property.value.as_ptr() as *const u32)});
                            println!("{}", value);
                        }
                    }
                    let console = VirtIOConsole::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
                    let mut console_mutex = CONSOLE.lock();
                    *console_mutex = Some(console);
                    println!("Console loaded.");
                }
                _ => {}
            }
        }
    }
}

pub fn get_plic(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/plic") {
        let reg = node.reg().unwrap().next().unwrap();
        let plic_reg = unsafe { &*((reg.starting_address as usize) as *const PLICRegs) };
        let plic = unsafe { Plic::new(NonNull::from_ref(plic_reg)) };
    }
}
