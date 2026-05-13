#![allow(dead_code)]

use spin::Mutex;

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

pub fn thread_yield() {
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
            li a7, 7
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
                li a7, 8
                ecall
                ",
            in("a0") thread_id,
        )
    }
}

pub struct Semaphore {}

impl Semaphore {
    pub fn set(counter_mutex: &Mutex<usize>, count: usize) {
        let mut counter = counter_mutex.lock();
        *counter = count;
    }
    pub fn get(counter_mutex: &Mutex<usize>) -> usize {
        let counter = counter_mutex.lock();
        return *counter;
    }
    pub fn wait(counter_mutex: &Mutex<usize>) {
        loop {
            let mut counter = counter_mutex.lock();
            if *counter > 0 {
                *counter = *counter - 1;
                break;
            } else {
                thread_yield();
            }
        }
    }
    pub fn post(counter_mutex: &Mutex<usize>) {
        let mut counter = counter_mutex.lock();
        *counter = *counter + 1;
        return;
    }
}
