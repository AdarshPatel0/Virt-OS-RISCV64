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
