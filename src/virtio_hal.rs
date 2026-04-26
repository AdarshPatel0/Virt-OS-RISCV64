extern crate alloc;

use alloc::alloc::{alloc, dealloc};
use core::{alloc::Layout, ptr::NonNull};
use virtio_drivers::*;

pub struct VirtIOHal {}

unsafe impl Hal for VirtIOHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, core::ptr::NonNull<u8>) {
        let allocation_pointer = unsafe { alloc(Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap()) };
        let safe_pointer = NonNull::new(allocation_pointer).unwrap();
        return (allocation_pointer as u64, safe_pointer);
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: core::ptr::NonNull<u8>, pages: usize) -> i32 {
        unsafe {
            dealloc(paddr as *mut u8, Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap());
        }
        return 0;
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: core::ptr::NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        unsafe { buffer.as_ref().as_ptr() as u64 }
    }

    unsafe fn unshare(_paddr: PhysAddr, _bufferr: core::ptr::NonNull<[u8]>, _direction: BufferDirection) {
        {}
    }
}