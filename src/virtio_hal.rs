extern crate alloc;

use alloc::alloc::{alloc_zeroed, dealloc};
use core::{alloc::Layout, ptr::NonNull};
use virtio_drivers::{BufferDirection, Hal, PAGE_SIZE, PhysAddr};

pub struct VirtIOHal {}

unsafe impl Hal for VirtIOHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, core::ptr::NonNull<u8>) {
        let size = pages * PAGE_SIZE;
        let layout = Layout::from_size_align(size, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!("VirtIOHal: dma_alloc failed for {} pages", pages);
        }
        let vaddr = NonNull::new(ptr).unwrap();
        let paddr = ptr as PhysAddr;
        (paddr,vaddr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: core::ptr::NonNull<u8>, pages: usize) -> i32 {
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        unsafe { dealloc(paddr as *mut u8, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).expect("MMIO paddr was null")
    }

    unsafe fn share(buffer: core::ptr::NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let paddr = buffer.as_ptr() as *mut u8 as PhysAddr;

        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

        paddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _bufferr: core::ptr::NonNull<[u8]>, _direction: BufferDirection) {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }
}
