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

use crate::virtio_hal;

pub static DISK: Mutex<Option<VirtIOBlk<virtio_hal::VirtIOHal, MmioTransport<'static>>>> = Mutex::new(None);
pub static CONSOLE: Mutex<Option<VirtIOConsole<virtio_hal::VirtIOHal, MmioTransport<'static>>>> = Mutex::new(None);

pub fn load_drivers(device_tree: Fdt) {
    for node in device_tree.find_all_nodes("/soc/virtio_mmio") {
        let reg = node.reg().unwrap().next().unwrap();
        let mmio_device_address = reg.starting_address as usize;
        let mmio_size = reg.size.unwrap();

        let header = NonNull::new(mmio_device_address as *mut VirtIOHeader).unwrap();

        match unsafe { MmioTransport::new(header, mmio_size) } {
            Ok(transport) => {
                let device_type = transport.device_type();
                match device_type {
                    DeviceType::Network => todo!(),
                    DeviceType::Block => {
                        let mut disk = DISK.lock();
                        if disk.is_none() {
                            *disk = Some(VirtIOBlk::<virtio_hal::VirtIOHal, _>::new(transport).unwrap());
                        }
                    }
                    DeviceType::Console => {
                        let mut console = CONSOLE.lock();
                        if console.is_none() {
                            *console = Some(VirtIOConsole::<virtio_hal::VirtIOHal, _>::new(transport).unwrap());
                        }
                    }
                    DeviceType::EntropySource => todo!(),
                    DeviceType::MemoryBallooning => todo!(),
                    DeviceType::IoMemory => todo!(),
                    DeviceType::Rpmsg => todo!(),
                    DeviceType::ScsiHost => todo!(),
                    DeviceType::_9P => todo!(),
                    DeviceType::Mac80211 => todo!(),
                    DeviceType::RprocSerial => todo!(),
                    DeviceType::VirtioCAIF => todo!(),
                    DeviceType::MemoryBalloon => todo!(),
                    DeviceType::GPU => todo!(),
                    DeviceType::Timer => todo!(),
                    DeviceType::Input => todo!(),
                    DeviceType::Socket => todo!(),
                    DeviceType::Crypto => todo!(),
                    DeviceType::SignalDistributionModule => todo!(),
                    DeviceType::Pstore => todo!(),
                    DeviceType::IOMMU => todo!(),
                    DeviceType::Memory => todo!(),
                    DeviceType::Sound => todo!(),
                }
            }
            Err(_) => {}
        }
    }
}
