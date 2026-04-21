pub fn exit() -> ! {
    unsafe {
        core::arch::asm!(
            "
            li a7, 0
            ecall
            "
        )
    }
    loop {}
}

pub fn get_hart_id() -> usize {
    let mut hart_id: usize = 0;
    unsafe {
        core::arch::asm!(
            "
            li a7, 2
            ecall
            ",
            out("a0") hart_id,
        )
    }
    return hart_id;
}
