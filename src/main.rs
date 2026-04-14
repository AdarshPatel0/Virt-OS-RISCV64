#![no_main]
#![no_std]

mod device_tree_utils;
mod ecall;
mod panic_handler;
mod system_utils;
mod thread;
mod timer_interrupt;
mod trap_handler;

unsafe extern "C" {
    static _kernel_end: u8;
}

#[global_allocator]
pub static HEAP: buddy_system_allocator::LockedHeap<32> = buddy_system_allocator::LockedHeap::<32>::empty();

core::arch::global_asm!(
    r#"
    .section .text.entry
    .globl _start
    _start:
        la sp, _stack_top
        j kmain
    "#
);

fn func() {
    unsafe {
        core::arch::asm!(
            "
            li a7, 1
            li a0, 2
            ecall
            "
        )
    }
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(_hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let device_tree = device_tree_utils::read_device_tree_data(device_tree_binary_ptr).expect("Failed to read device tree.");

    let (system_memory_base_address, system_memory_amount) = device_tree_utils::get_system_memory_info(&device_tree).expect("Failed to get system memory amount.");

    let kernel_end_address = core::ptr::addr_of!(_kernel_end) as usize;

    unsafe {
        let mut heap = HEAP.lock();
        heap.add_to_heap(kernel_end_address, system_memory_base_address + system_memory_amount);
        drop(heap);
    };

    timer_interrupt::set_time_quanta(1_000_000);

    unsafe {
        riscv::register::stvec::write(riscv::register::stvec::Stvec::new(trap_handler::entry::trap_handler_entry as *const u8 as usize, riscv::register::stvec::TrapMode::Direct));
        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);
    }

    thread::create_thread(func as *const u8 as usize, false);
    thread::create_thread(func as *const u8 as usize, false);

    timer_interrupt::update_timer();

    loop {
        riscv::asm::wfi();
    }
}
