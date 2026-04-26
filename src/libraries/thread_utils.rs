#![allow(dead_code)]

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

pub fn thread_create<T>(entry: usize, stack_size: usize, arguments: &T) -> usize {
    let mut tid: usize = 0;
    unsafe {
        core::arch::asm!(
            "
            li a7, 2
            ecall
            ",
            in("a0") entry,
            in("a1") stack_size,
            in("a2") arguments as *const T as usize,
            in("a3") core::mem::size_of::<T>(),
            lateout("a0") tid
        )
    }
    return tid;
}

pub fn thread_wait(thread_id: usize) {
    unsafe {
        core::arch::asm!(
            "
                li a7, 3
                ecall
                ",
            in("a0") thread_id,
        )
    }
}
