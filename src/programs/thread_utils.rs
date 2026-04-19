pub fn exit() {
    unsafe {
        core::arch::asm!(
            "
            li a7, 0
            ecall
            "
        )
    }
}

pub fn r#yield() {
    unsafe {
        core::arch::asm!(
            "
            li a7, 1
            ecall
            "
        )
    }
}
