#![no_main]
#![no_std]

mod device_tree_utils;
mod panic_handler;
mod system_utils;
mod trap_handler;
mod timer_interrupt;
mod ecall;
mod thread;


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


#[unsafe(no_mangle)]
pub extern "C" fn kmain(_hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let device_tree = device_tree_utils::read_device_tree_data(device_tree_binary_ptr).expect("Failed to read device tree.");

    let (_system_memory_base_address, _system_memory_amount) = device_tree_utils::get_system_memory_info(&device_tree).expect("Failed to get system memory amount.");

    let _kernel_end_address = core::ptr::addr_of!(_kernel_end) as usize;

    unsafe {
        riscv::register::stvec::write(riscv::register::stvec::Stvec::new(trap_handler::entry::trap_handler_entry as *const u8 as usize, riscv::register::stvec::TrapMode::Direct));
        riscv::interrupt::enable();
    }

    timer_interrupt::init(1_000_000);

    loop {}
}
