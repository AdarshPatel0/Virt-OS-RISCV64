pub fn thread_exit() -> ! {
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

pub fn thread_release() {
    unsafe {
        core::arch::asm!(
            "
            li a7, 1
            ecall
            "
        )
    }
}

pub fn thread_create(entry: usize) -> usize {
    let mut tid: usize = 0;
    unsafe {
        core::arch::asm!(
            "
            li a7, 2
            ecall
            ",
            in("a0") entry,
            lateout("a0") tid
        )
    }
    return tid;
}

pub fn thread_wait(thread_id: usize) {
    loop {
        let mut thread_exists: usize = 0;
        unsafe {
            core::arch::asm!(
                "
                li a7, 3
                ecall
                ",
                in("a0") thread_id,
                lateout("a0") thread_exists
            )
        }
        if thread_exists == 1 {
            thread_release();
        } else {
            break;
        }
    }
}
