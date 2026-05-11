use alloc::sync::Arc;
use core::{num::NonZero, ptr::NonNull};
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

pub static PLIC: Mutex<Option<Plic>> = Mutex::new(None);
pub static CONSOLE: Mutex<Option<VirtIOConsole<VirtIOHal, MmioTransport>>> = Mutex::new(None);
pub static BLOCK_DEVICES: Mutex<slab::Slab<Arc<Mutex<VirtIOBlk<VirtIOHal, MmioTransport>>>>> = Mutex::new(slab::Slab::new());

pub fn load_drivers(device_tree: &Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
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
                        let interrupts = node.interrupts().unwrap().next().unwrap();
                        let virtio_irq = NonZero::new(interrupts as u32).unwrap();
                        if let Some(plic) = PLIC.lock().as_mut() {
                            plic.set_priority(virtio_irq, 1);
                            plic.enable(virtio_irq, 1);
                            plic.set_threshold(1, 0);
                        }
                        *global_console = Some(console);
                    } else {
                        panic!("Multiple console devices detected.");
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn load_plic(device_tree: &Fdt) {
    for node in device_tree.find_all_nodes("/soc/plic") {
        let mut regs = node.reg().unwrap();
        let reg = regs.next().unwrap();
        let plic = unsafe { Plic::new(NonNull::new(reg.starting_address as *mut PLICRegs).unwrap()) };
        *PLIC.lock() = Some(plic);
    }
}
