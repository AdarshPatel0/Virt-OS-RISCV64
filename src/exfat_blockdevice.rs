use alloc::sync::Arc;
use exfat_slim::blocking::BlockDevice;
use spin::Mutex;
use virtio_drivers::{Hal, device::blk::VirtIOBlk, transport::Transport};

pub struct ExFATBlockDevice<H: Hal, T: Transport> {
    block_device_mutex: Arc<Mutex<VirtIOBlk<H, T>>>
}

impl <H: Hal, T: Transport> BlockDevice<512> for ExFATBlockDevice<H, T> {
    type Error = virtio_drivers::Error;

    type Align = aligned::Aligned<512,>;

    fn read(
        &mut self,
        block_address: u32,
        data: &mut [Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn write(
        &mut self,
        block_address: u32,
        data: &[Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn size(&mut self) -> Result<u64, Self::Error> {
        todo!()
    }
}