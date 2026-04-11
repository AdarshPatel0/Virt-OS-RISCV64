use riscv::{interrupt::supervisor::Interrupt, register::time};
use sbi::timer;

static mut TIME_QUANTA: u64 = 0;

#[riscv_rt::core_interrupt(Interrupt::SupervisorTimer)]
fn supervisor_timer_interrupt() {
    timer::set_timer(time::read64() + unsafe { TIME_QUANTA }).unwrap();
    return;
}

pub fn init(time_quanta: u64) {
    unsafe {
        TIME_QUANTA = time_quanta;
        riscv::interrupt::supervisor::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);
    }
    timer::set_timer(time::read64() + time_quanta).unwrap();
}
