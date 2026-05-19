use alloc::sync::Arc;
use rsext4::{BLOCK_SIZE, BlockDevice, Ext4Error, Ext4Result, Ext4Timestamp};
use spin::Mutex;
use virtio_drivers::{
    Hal,
    device::blk::{SECTOR_SIZE, VirtIOBlk},
    transport::Transport,
};

use crate::{devices::RTC, libraries::console_utils::print};

pub struct Ext4BlockDevice<H: Hal, T: Transport> {
    block_device: Arc<Mutex<VirtIOBlk<H, T>>>,
    sectors_per_block: usize,
}

impl<H: Hal, T: Transport> Ext4BlockDevice<H, T> {
    pub fn new(block_device: Arc<Mutex<VirtIOBlk<H, T>>>) -> Self {
        return Self {
            block_device,
            sectors_per_block: BLOCK_SIZE / SECTOR_SIZE,
        };
    }
}

impl<H: Hal, T: Transport> BlockDevice for Ext4BlockDevice<H, T> {
    fn write(&mut self, buffer: &[u8], block_id: rsext4::bmalloc::AbsoluteBN, _count: u32) -> rsext4::Ext4Result<()> {
        let mut block_device = self.block_device.lock();
        if let Err(_) = block_device.write_blocks(block_id.as_usize()? * self.sectors_per_block, buffer) {
            return Ext4Result::Err(Ext4Error::io());
        }
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8], block_id: rsext4::bmalloc::AbsoluteBN, _count: u32) -> rsext4::Ext4Result<()> {
        let mut block_device = self.block_device.lock();
        if let Err(err) = block_device.read_blocks(block_id.as_usize()? * self.sectors_per_block, buffer) {
            print!("{}",err);
            return Ext4Result::Err(Ext4Error::io());
        }
        Ok(())
    }

    fn open(&mut self) -> rsext4::Ext4Result<()> {
        Ok(())
    }

    fn close(&mut self) -> rsext4::Ext4Result<()> {
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        return self.block_device.lock().capacity() / self.sectors_per_block as u64;
    }

    fn current_time(&self) -> rsext4::Ext4Result<rsext4::Ext4Timestamp> {
        let timestamp = match RTC.get() {
            Some(rtc) => rtc.get_unix_timestamp(),
            None => 0,
        };
        return Ok(Ext4Timestamp::new(timestamp as i64, 0));
    }

    fn block_size(&self) -> u32 {
        return SECTOR_SIZE as u32;
    }

    fn flush(&mut self) -> rsext4::Ext4Result<()> {
        if let Err(_) = self.block_device.lock().flush() {
            return Ext4Result::Err(Ext4Error::io());
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn is_readonly(&self) -> bool {
        false
    }
}