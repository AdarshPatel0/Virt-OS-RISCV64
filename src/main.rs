#![no_main]
#![no_std]

extern crate alloc;

mod context;
mod device_tree_utils;
mod drivers;
mod ecall;
mod hart;
mod libraries;
mod panic_handler;
mod print;
mod programs;
mod thread;
mod timer_interrupt;
mod trap_handler;
mod virtio_fatfs;
mod virtio_hal;

unsafe extern "C" {
    static _kernel_end: u8;
    static _stack_top: u8;
}

#[global_allocator]
pub static HEAP: buddy_system_allocator::LockedHeap<32> = buddy_system_allocator::LockedHeap::<32>::empty();

pub static DEVICE_TREE_PTR: spin::Once<usize> = spin::Once::new();

core::arch::global_asm!(
    r#"
    .section .text.entry
    .globl _start
    _start:
        la sp, _stack_top
        j kmain
    "#
);

#[unsafe(no_mangle)]
extern "C" fn kmain(hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let _ = *DEVICE_TREE_PTR.call_once(|| device_tree_binary_ptr);
    let device_tree = device_tree_utils::get_device_tree(device_tree_binary_ptr);

    let device_memory = device_tree.memory().regions().next().expect("Failed to get device memory region");
    let system_memory_base_address = device_memory.starting_address as usize;
    let system_memory_amount = device_memory.size.expect("Failed to get memory amount");
    let kernel_end_address = core::ptr::addr_of!(_kernel_end) as usize;

    unsafe {
        let mut heap = HEAP.lock();
        heap.init(kernel_end_address, system_memory_amount - (kernel_end_address - system_memory_base_address));
        drop(heap);
    };

    drivers::load_plic(&device_tree);
    drivers::load_drivers(&device_tree);

    if let Some(block_device_mutex) = drivers::BLOCK_DEVICES.lock().get_mut(0) {
        let file_system: crate::programs::shell::FatFileSystem = fatfs::FileSystem::new(crate::virtio_fatfs::FatFsBlockDevice::new(block_device_mutex.clone()), fatfs::FsOptions::new()).unwrap();
        let raw_slice = unsafe {
            let size = size_of::<crate::programs::shell::FatFileSystem>();
            let ptr = &file_system as *const crate::programs::shell::FatFileSystem as *const u8;
            core::slice::from_raw_parts(ptr, size)
        };
        thread::create_thread(programs::shell::new as *const u8 as usize, false, 16384, raw_slice);
        core::mem::forget(file_system);
    }

    timer_interrupt::set_time_quanta(10_000_000);

    for cpu in device_tree.cpus() {
        let id = cpu.ids().first();
        if id == hart_id {
            continue;
        }
        hart::initialize_hart(id, id == hart_id);
    }

    hart::initialize_hart(hart_id, true);

    loop {
        riscv::asm::wfi();
    }
}
