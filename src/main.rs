#![no_main]
#![no_std]

mod device_tree_utils;
mod panic_handler;
mod system_utils;

use buddy_system_allocator::LockedHeap;

const TIME_QUANTA: usize = 1_000_000;

unsafe extern "C" {
    static kernel_end: u8;
}

#[unsafe(no_mangle)]
extern "C" fn say_hello() -> ! {
    let character: u8 = b'B';
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") character,
        );
    }
    loop {}
}

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

#[riscv_rt::entry]
fn main(_hart_id: usize, device_tree_binary_ptr: usize) -> ! {
    let device_tree = device_tree_utils::read_device_tree_data(device_tree_binary_ptr).expect("Failed to read device tree.");

    let (system_memory_base_address, system_memory_amount) = device_tree_utils::get_system_memory_info(&device_tree).expect("Failed to get system memory amount.");

    let kernel_end_address = core::ptr::addr_of!(kernel_end) as usize;

    unsafe {
        HEAP.lock().add_to_heap(kernel_end_address, system_memory_base_address + system_memory_amount);
    }

    let time = riscv::register::time::read();
    sbi::timer::set_timer((time + TIME_QUANTA) as u64).unwrap();

    unsafe {
        riscv::interrupt::supervisor::enable();
        riscv::interrupt::supervisor::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);
    }

    let mut heap = HEAP.lock();

    const SHELL_STACK_SIZE: usize = 4096;
    match (*heap).alloc(core::alloc::Layout::from_size_align(SHELL_STACK_SIZE, 16).unwrap()) {
        Ok(shell_stack_start) => unsafe {
            riscv::register::sepc::write(say_hello as *const () as usize);
            riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
            core::arch::asm!("mv sp, {}", in(reg) shell_stack_start.as_ptr() as usize + SHELL_STACK_SIZE);
            core::arch::asm!("sret");
        },
        Err(_) => system_utils::shutdown(),
    }

    loop {}
}

#[riscv_rt::core_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer)]
fn supervisor_timer() {
    riscv::interrupt::supervisor::disable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);

    let time = riscv::register::time::read();
    sbi::timer::set_timer((time + TIME_QUANTA) as u64).unwrap();
    println!("TINT");

    unsafe {
        riscv::interrupt::supervisor::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);
    }
    return;
}

#[unsafe(export_name = "ExceptionHandler")]
fn custom_exception_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    let cause = riscv::register::scause::read();
    if cause.is_exception() {
        match cause.code() {
            8 => {
                let character: u8 = trap_frame.a0 as u8;
                println!("{}", character as char);
            }
            _ => {}
        }
    }
    loop {}
}
