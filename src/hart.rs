use alloc::alloc::*;
use riscv::register::sscratch;
use sbi::PhysicalAddress;
use sbi::hart_state_management::*;

use crate::timer_interrupt;
use crate::trap_handler;

extern crate alloc;

const STACK_SIZE: usize = 4096;

#[repr(C)]
struct hart_info {
    hart_id: usize,
    stack_start: usize,
    stack_size: usize,
}

#[unsafe(naked)]
extern "C" fn hart_startup_entry() {
    core::arch::naked_asm!(
        "
        ld sp, 1*8(a1)
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
        let hart_stack = alloc(Layout::from_size_align(STACK_SIZE, 16).unwrap());

        let hart_info = alloc(Layout::new::<hart_info>()) as *mut hart_info;

        hart_info.write(hart_info {
            hart_id,
            stack_start: hart_stack as usize,
            stack_size: STACK_SIZE,
        });

        hart_start(hart_id, PhysicalAddress::new(hart_startup_entry as *const u8 as usize), hart_info as usize).unwrap();
    }
}
