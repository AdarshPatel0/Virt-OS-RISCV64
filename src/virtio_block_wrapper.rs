use rsext4::blockdev::BlockDevice;
use virtio_drivers::{Hal, device::blk::VirtIOBlk, transport::Transport};

pub struct VirtioBlockWrapper<H: Hal, T: Transport> {
    pub block_device: VirtIOBlk<H, T>,
}

impl<H: Hal, T: Transport> VirtioBlockWrapper<H, T> {
    pub fn new(block_device: VirtIOBlk<H, T>) -> Self {
        return VirtioBlockWrapper { block_device };
    }
}

impl<H: Hal, T: Transport> BlockDevice for VirtioBlockWrapper<H, T> {
    fn write(&mut self, buffer: &[u8], block_id: rsext4::bmalloc::AbsoluteBN, _count: u32) -> rsext4::Ext4Result<()> {
        self.block_device.write_blocks(block_id.as_usize().unwrap(), buffer).unwrap();
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8], block_id: rsext4::bmalloc::AbsoluteBN, _count: u32) -> rsext4::Ext4Result<()> {
        self.block_device.read_blocks(block_id.as_usize().unwrap(), buffer).unwrap();
        Ok(())
    }

    fn open(&mut self) -> rsext4::Ext4Result<()> {
        Ok(())
    }

    fn close(&mut self) -> rsext4::Ext4Result<()> {
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        return self.block_device.capacity();
    }

    fn current_time(&self) -> rsext4::Ext4Result<rsext4::Ext4Timestamp> {
        Ok(rsext4::Ext4Timestamp { sec: 1767225600, nsec: 0 })
    }

    fn block_size(&self) -> u32 {
        return virtio_drivers::device::blk::SECTOR_SIZE as u32;
    }
}
