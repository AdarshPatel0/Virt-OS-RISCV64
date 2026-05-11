use alloc::sync::Arc;
use fatfs::{IoBase, Read, Seek, Write};
use spin::Mutex;
use virtio_drivers::{
    Hal,
    device::blk::{SECTOR_SIZE, VirtIOBlk},
    transport::Transport,
};

pub struct FatFsBlockDevice<H: Hal, T: Transport> {
    block_device_mutex: Arc<Mutex<VirtIOBlk<H, T>>>,
    cursor: u64,
    sector_buffer: [u8; SECTOR_SIZE],
}

impl<H: Hal, T: Transport> FatFsBlockDevice<H, T> {
    pub fn new(block_device_mutex: Arc<Mutex<VirtIOBlk<H, T>>>) -> Self {
        return FatFsBlockDevice {
            block_device_mutex,
            cursor: 0,
            sector_buffer: [0u8; SECTOR_SIZE],
        };
    }
}

impl<H: Hal, T: Transport> IoBase for FatFsBlockDevice<H, T> {
    type Error = ();
}

impl<H: Hal, T: Transport> Read for FatFsBlockDevice<H, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut block_device = self.block_device_mutex.lock();
        let mut buffer_cursor = 0;

        while buffer_cursor < buf.len() {
            let current_block = (self.cursor / SECTOR_SIZE as u64) as usize;
            let offset = (self.cursor % SECTOR_SIZE as u64) as usize;
            block_device.read_blocks(current_block, &mut self.sector_buffer).unwrap();

            let sector_bytes_remaining = SECTOR_SIZE - offset;
            let bytes_to_read = { if sector_bytes_remaining < buf.len() - buffer_cursor { sector_bytes_remaining } else { buf.len() - buffer_cursor } };

            buf[buffer_cursor..buffer_cursor + bytes_to_read].copy_from_slice(&self.sector_buffer[offset..offset + bytes_to_read]);
            buffer_cursor += bytes_to_read;
            self.cursor += bytes_to_read as u64;
        }

        return Ok(buffer_cursor);
    }
}

impl<H: Hal, T: Transport> Write for FatFsBlockDevice<H, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut block_device = self.block_device_mutex.lock();
        let mut buffer_cursor = 0;

        while buffer_cursor < buf.len() {
            let current_block = (self.cursor / SECTOR_SIZE as u64) as usize;
            let offset = (self.cursor % SECTOR_SIZE as u64) as usize;
            block_device.read_blocks(current_block, &mut self.sector_buffer).unwrap();

            let sector_bytes_remaining = SECTOR_SIZE - offset;
            let bytes_to_write = { if sector_bytes_remaining < buf.len() - buffer_cursor { sector_bytes_remaining } else { buf.len() - buffer_cursor } };

            self.sector_buffer[offset..offset + bytes_to_write].copy_from_slice(&buf[buffer_cursor..buffer_cursor + bytes_to_write]);

            block_device.write_blocks(current_block, &self.sector_buffer).unwrap();

            buffer_cursor += bytes_to_write;
            self.cursor += bytes_to_write as u64;
        }

        return Ok(buffer_cursor);
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        return Ok(());
    }
}

impl<H: Hal, T: Transport> Seek for FatFsBlockDevice<H, T> {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        let block_device = self.block_device_mutex.lock();
        match pos {
            fatfs::SeekFrom::Start(offset) => self.cursor = offset,
            fatfs::SeekFrom::End(offset) => {
                let total_size = block_device.capacity() * SECTOR_SIZE as u64;
                self.cursor = (total_size as i64 + offset) as u64
            }
            fatfs::SeekFrom::Current(offset) => self.cursor = (self.cursor as i64 + offset) as u64,
        }
        return Ok(self.cursor);
    }
}
