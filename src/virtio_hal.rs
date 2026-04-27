extern crate alloc;

use alloc::alloc::{alloc, dealloc};
use core::{alloc::Layout, ptr::NonNull};
use virtio_drivers::*;

pub struct VirtIOHal {}

unsafe impl Hal for VirtIOHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, core::ptr::NonNull<u8>) {
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc(layout) };

        let nn = NonNull::new(ptr).expect("dma_alloc failed");
        (ptr as PhysAddr, nn)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: core::ptr::NonNull<u8>, pages: usize) -> i32 {
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        unsafe { dealloc(paddr as *mut u8, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: core::ptr::NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        buffer.as_ptr() as *mut u8 as PhysAddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _bufferr: core::ptr::NonNull<[u8]>, _direction: BufferDirection) {
        {}
    }
}
