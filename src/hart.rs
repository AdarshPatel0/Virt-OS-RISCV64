use crate::timer_interrupt;
use crate::trap_handler;

extern crate alloc;

pub const STACK_SIZE: usize = 4096;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct HartInfo {
    pub stack_start_address: usize,
    pub stack_size: usize,
    pub hart_id: usize,
    pub current_thread: Option<usize>,
}

#[unsafe(naked)]
extern "C" fn hart_startup_entry() {
    core::arch::naked_asm!(
        "
        csrw sscratch, a1
        mv sp, a1
        mv a0, a1
        call hart_startup
        "
    )
}

#[unsafe(no_mangle)]
extern "C" fn hart_startup(_hart_info: &mut HartInfo) -> ! {
    unsafe {
        use riscv::{
            interrupt,
            register::{self, stvec},
        };
        register::stvec::write(riscv::register::stvec::Stvec::new(trap_handler::entry::trap_handler_entry as *const u8 as usize, stvec::TrapMode::Direct));
        interrupt::enable();
        interrupt::enable_interrupt(interrupt::Interrupt::SupervisorTimer);
    }

    timer_interrupt::update_timer();
    loop {}
}

pub fn initialize_hart(hart_id: usize, current_hart: bool) {
    use alloc::alloc::*;
    use core::mem::size_of;
    use sbi::PhysicalAddress;
    use sbi::hart_state_management::*;

    unsafe {
        let hart_stack = alloc(Layout::from_size_align(STACK_SIZE, 16).unwrap());

        let hart_info_ptr = (hart_stack as usize + STACK_SIZE - size_of::<HartInfo>()) as *mut HartInfo;

        hart_info_ptr.write(HartInfo {
            stack_start_address: hart_stack as usize,
            stack_size: STACK_SIZE,
            hart_id,
            current_thread: None,
        });

        if !current_hart {
            hart_start(hart_id, PhysicalAddress::new(hart_startup_entry as *const u8 as usize), hart_info_ptr as usize).unwrap();
        } else {
            use riscv::interrupt;
            interrupt::enable();
            interrupt::enable_interrupt(interrupt::Interrupt::SupervisorTimer);
            timer_interrupt::update_timer();
            hart_suspend(SuspendType::DefaultNonRetentive {
                resume_address: PhysicalAddress::new(hart_startup_entry as *const u8 as usize),
                opaque: hart_info_ptr as usize,
            })
            .unwrap();
        }
    }
}
