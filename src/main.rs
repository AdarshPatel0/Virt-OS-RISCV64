#![no_main]
#![no_std]

mod device_tree_utils;
mod panic_handler;
mod system_utils;
mod timer_interrupt;
mod thread;

use buddy_system_allocator::LockedHeap;

unsafe extern "C" {
    static kernel_end: u8;
}

#[global_allocator]
pub static HEAP: LockedHeap<32> = LockedHeap::empty();

#[riscv_rt::entry]
fn main(_hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let device_tree = device_tree_utils::read_device_tree_data(device_tree_binary_ptr).expect("Failed to read device tree.");

    let (system_memory_base_address, system_memory_amount) = device_tree_utils::get_system_memory_info(&device_tree).expect("Failed to get system memory amount.");

    let kernel_end_address = core::ptr::addr_of!(kernel_end) as usize;

    unsafe {
        HEAP.lock().add_to_heap(kernel_end_address, system_memory_base_address + system_memory_amount);
    }

    unsafe {
        riscv::interrupt::supervisor::enable();
    }

    timer_interrupt::init(1_000_000);

    loop {}
}