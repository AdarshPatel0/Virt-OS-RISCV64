use core::ptr::NonNull;
use fdt::Fdt;
use riscv_plic::{PLICRegs, Plic};
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
pub static PLIC: Mutex<Plic> = unsafe { Mutex::new(Plic::new(NonNull::dangling())) };

pub fn load_drivers(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
        let reg = node.reg().unwrap().next().unwrap();
        let mmio_device_address = reg.starting_address as usize;
        let mmio_size = reg.size.unwrap();

        let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

        if let Ok(transport) = unsafe { MmioTransport::new(header, mmio_size) } {
            match transport.device_type() {
                DeviceType::Block => {
                    if let Some(property) = node.property("interrupt-parent") {
                        println!("{}", unsafe { u32::from_be(*(property.value.as_ptr() as *const u32)) });
                    }
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

pub fn load_plic(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/plic") {
        if let Some(mut registers) = node.reg() {
            if let Some(register) = registers.next() {
                let plic_ptr = NonNull::new(register.starting_address as *mut PLICRegs).unwrap();
                let plic = unsafe { Plic::new(plic_ptr) };
                *PLIC.lock() = plic;
            }
        }
    }
}

fn insert_block_device(transport: MmioTransport<'static>) {
    let block_device = VirtIOBlk::<VirtIOHal, MmioTransport<'_>>::new(transport).unwrap();
    BLOCK_DEVICES.lock().insert(Mutex::new(block_device));
}
