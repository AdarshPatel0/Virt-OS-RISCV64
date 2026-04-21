use crate::thread;

pub mod context;

extern crate alloc;

static STACK_SIZE: usize = 4096;

#[allow(unused)]
pub struct Thread {
    pub context: context::Context,
    pub stack: alloc::boxed::Box<[u8]>,
    pub dead: bool,
}

static mut THREADS: slab::Slab<Thread> = slab::Slab::new();
static mut QUEUE: alloc::collections::VecDeque<usize> = alloc::collections::VecDeque::new();
static mut CURRENT: Option<usize> = None;

#[allow(unused)]
pub fn create_thread(entry: usize, privileged: bool) -> usize {
    let thread_stack = alloc::vec![0 as u8; STACK_SIZE].into_boxed_slice();
    let stack_top = thread_stack.as_ptr() as usize + STACK_SIZE;

    let mut thread_context = context::Context::default();
    thread_context.sp = stack_top;
    thread_context.sepc = entry;

    let mut sstatus = riscv::register::sstatus::read();

    if privileged {
        sstatus.set_spp(riscv::register::sstatus::SPP::Supervisor);
    } else {
        sstatus.set_spp(riscv::register::sstatus::SPP::User);
    }

    sstatus.set_spie(true);

    thread_context.sstatus = sstatus.bits();

    let thread = Thread {
        context: thread_context,
        stack: thread_stack,
        dead: false,
    };

    let id = unsafe {
        let threads = &mut *(&raw mut THREADS);
        let id = threads.insert(thread);

        let queue = &mut *(&raw mut QUEUE);
        queue.push_back(id);
        id
    };
    return id;
}

pub fn delete_thread(id: usize) -> bool {
    unsafe {
        let threads = &mut *(&raw mut THREADS);
        match threads.get_mut(id) {
            Some(thread) => {
                thread.dead = true;
                true
            }
            None => false,
        }
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wait() {
    core::arch::naked_asm!(
        "
        wait_start:
            wfi
            j   wait_start
        "
    );
}

pub fn schedule(context: &mut thread::context::Context) {
    unsafe {
        let threads = &mut *(&raw mut THREADS);
        let queue = &mut *(&raw mut QUEUE);
        if let Some(current_thread_id) = CURRENT {
            if let Some(current_thread) = threads.get_mut(current_thread_id) {
                queue.push_back(current_thread_id);
                current_thread.context = *context;
            }
        }
        loop {
            match queue.pop_front() {
                Some(new_thread_id) => {
                    if let Some(new_thread) = threads.get(new_thread_id) {
                        if new_thread.dead == true {
                            threads.remove(new_thread_id);
                            continue;
                        }
                    }
                    if let Some(new_thread) = threads.get(new_thread_id) {
                        *context = new_thread.context;
                        CURRENT = Some(new_thread_id);
                        return;
                    }
                }
                None => {
                    CURRENT = None;
                    context.sepc = wait as *const u8 as usize;
                    let mut sstatus = riscv::register::sstatus::read();
                    sstatus.set_spie(true);
                    sstatus.set_spp(riscv::register::sstatus::SPP::Supervisor);
                    context.sstatus = sstatus.bits();
                    return;
                }
            }
        }
    }
}

pub fn get_current_thread() -> Option<usize> {
    unsafe { CURRENT }
}
