#![no_main]
#![no_std]

mod device_tree_utils;
mod panic_handler;
mod system_utils;

use buddy_system_allocator::LockedHeap;

unsafe extern "C" {
    static _kernel_end: u8;
}

core::arch::global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "    la sp, _stack_top",
    "    j kmain"
);

#[global_allocator]
pub static HEAP: LockedHeap<32> = LockedHeap::empty();

#[unsafe(no_mangle)]
pub extern "C" fn kmain(_hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let device_tree = device_tree_utils::read_device_tree_data(device_tree_binary_ptr).expect("Failed to read device tree.");

    let (system_memory_base_address, system_memory_amount) = device_tree_utils::get_system_memory_info(&device_tree).expect("Failed to get system memory amount.");

    let kernel_end_address = core::ptr::addr_of!(_kernel_end) as usize;

    unsafe {
        HEAP.lock().add_to_heap(kernel_end_address, system_memory_base_address + system_memory_amount);
    };

    

    system_utils::shutdown()
}