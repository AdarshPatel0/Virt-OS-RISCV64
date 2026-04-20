#![no_main]
#![no_std]

mod device_tree_utils;
mod ecall;
mod panic_handler;
mod print_macros;
mod programs;
mod thread;
mod timer_interrupt;
mod trap_handler;

unsafe extern "C" {
    static _kernel_end: u8;
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
extern "C" fn kmain(_hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let _ = *DEVICE_TREE_PTR.call_once(|| device_tree_binary_ptr);

    let device_tree = device_tree_utils::get_device_tree(device_tree_binary_ptr);
    let device_memory = device_tree.memory().regions().next().expect("Failed to get device memory region");
    let system_memory_base_address = device_memory.starting_address as usize;
    let system_memory_amount = device_memory.size.expect("Failed to get memory amount");
    let kernel_end_address = core::ptr::addr_of!(_kernel_end) as usize;

    unsafe {
        let mut heap = HEAP.lock();
        heap.add_to_heap(kernel_end_address, system_memory_base_address + system_memory_amount);
        drop(heap);
    };

    thread::create_thread(programs::shell::shell as *const u8 as usize, false);

    timer_interrupt::set_time_quanta(1_000_000);

    unsafe {
        riscv::register::stvec::write(riscv::register::stvec::Stvec::new(trap_handler::entry::trap_handler_entry as *const u8 as usize, riscv::register::stvec::TrapMode::Direct));
        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);
    }

    timer_interrupt::update_timer();

    loop {
        riscv::asm::wfi();
    }
}
