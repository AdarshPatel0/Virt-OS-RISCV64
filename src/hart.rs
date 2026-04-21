use alloc::alloc::*;
use riscv::register::sscratch;
use sbi::PhysicalAddress;
use sbi::hart_state_management::*;

use crate::timer_interrupt;
use crate::trap_handler;

extern crate alloc;

const STACK_SIZE: usize = 4096;

#[unsafe(naked)]
extern "C" fn hart_startup_entry() {
    core::arch::naked_asm!(
        "
        mv sp, a1
        addi sp, sp, -8
        sd a0, 0*8(sp)
        call hart_startup
        "
    )
}

#[unsafe(no_mangle)]
pub fn hart_startup(hart_id: usize) {
    unsafe {
        use riscv::{
            interrupt,
            register::{self, stvec},
        };
        register::sscratch::write(hart_id);
        register::stvec::write(riscv::register::stvec::Stvec::new(trap_handler::entry::trap_handler_entry as *const u8 as usize, stvec::TrapMode::Direct));
        interrupt::enable();
        interrupt::enable_interrupt(interrupt::Interrupt::SupervisorTimer);
    }

    timer_interrupt::update_timer();
    loop {}
}

pub fn initialize_hart(hart_id: usize) {
    unsafe {
        let cpu_stack = alloc(Layout::from_size_align(STACK_SIZE, 16).unwrap());
        hart_start(hart_id, PhysicalAddress::new(hart_startup_entry as *const u8 as usize), cpu_stack as usize).unwrap();
    }
}
