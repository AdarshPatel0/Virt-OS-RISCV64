use crate::timer_interrupt;
use crate::trap_handler;

extern crate alloc;

pub const STACK_SIZE: usize = 16384;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct HartInfo {
    pub hart_id: usize,
    pub current_thread_id: Option<usize>,
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
    loop {
        riscv::asm::wfi();
    }
}

pub fn initialize_hart(hart_id: usize, current_hart: bool) {
    use alloc::alloc::*;
    use core::mem::size_of;
    use sbi::PhysicalAddress;
    use sbi::hart_state_management::*;

    unsafe {
        let hart_stack_ptr = alloc(Layout::from_size_align(STACK_SIZE, 16).unwrap());

        let hart_stack_top = hart_stack_ptr as usize + STACK_SIZE - size_of::<HartInfo>();

        let hart_info_ptr = hart_stack_top as *mut HartInfo;

        *hart_info_ptr = HartInfo { hart_id, current_thread_id: None };

        if !current_hart {
            hart_start(hart_id, PhysicalAddress::new(hart_startup_entry as *const u8 as usize), hart_stack_top as usize as usize).unwrap();
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

pub fn get_hart_info_pointer() -> *mut HartInfo {
    let hart_info_ptr = riscv::register::sscratch::read();
    hart_info_ptr as *mut HartInfo
}
