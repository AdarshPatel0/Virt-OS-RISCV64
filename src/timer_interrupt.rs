static mut TIME_QUANTA: u64 = 0;

pub fn timer_interrupt() {
    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + unsafe { TIME_QUANTA }).unwrap();
    return;
}

pub fn init(time_quanta: u64) {
    unsafe {
        TIME_QUANTA = time_quanta;
        riscv::interrupt::enable_interrupt(riscv::interrupt::supervisor::Interrupt::SupervisorTimer);
    }
    timer_interrupt();
}
