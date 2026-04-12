#![no_main]
#![no_std]

mod device_tree_utils;
mod ecall;
mod panic_handler;
mod system_utils;
mod threads;
mod trap_handler;

use buddy_system_allocator::LockedHeap;

unsafe extern "C" {
    static _kernel_end: u8;
}

core::arch::global_asm!(
    r#"
    .section .text.entry
    .globl _start
    _start:
        la sp, _stack_top
        j kmain
    "#
);

#[global_allocator]
pub static HEAP: LockedHeap<32> = LockedHeap::empty();

fn user_thread() -> ! {
    let n = 2;
    unsafe {
        core::arch::asm!(
            "li a7, 1",
            "mv a0, {0}",
            "ecall",
            in(reg) n
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
        HEAP.lock().add_to_heap(kernel_end_address, system_memory_base_address + system_memory_amount);
    };

    unsafe {
        riscv::register::stvec::write(riscv::register::stvec::Stvec::new(trap_handler::trap_handler_entry as *const u8 as usize, riscv::register::stvec::TrapMode::Direct));

        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);

        let time = riscv::register::time::read64();
        sbi::timer::set_timer(time + 1_000_000).unwrap();
    }

    let stack_size: usize = 2048;
    let stack_start = HEAP.lock().alloc(core::alloc::Layout::from_size_align(stack_size, 16).unwrap()).unwrap();
    let stack_base = stack_start.as_ptr() as usize + stack_size;
    unsafe {
        riscv::register::sepc::write(user_thread as *const u8 as usize);
        core::arch::asm!(
            "mv sp, {0}",
            "sret",
            in(reg) stack_base
        );
    }

    loop {}
}
