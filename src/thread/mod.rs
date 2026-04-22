use crate::{hart, print::println, thread};

pub mod context;

extern crate alloc;

static STACK_SIZE: usize = 4096;

#[allow(unused)]
pub struct Thread {
    pub context: context::Context,
    pub stack: alloc::boxed::Box<[u8]>,
    pub dead: bool,
}

pub static THREADS: spin::Mutex<slab::Slab<Thread>> = spin::Mutex::new(slab::Slab::new());
static QUEUE: spin::Mutex<alloc::collections::VecDeque<usize>> = spin::Mutex::new(alloc::collections::VecDeque::new());

#[allow(unused)]
pub fn create_thread(entry: usize, privileged: bool, args: &[u8]) -> usize {
    let thread_stack = alloc::vec![0 as u8; STACK_SIZE].into_boxed_slice();
    let stack_top = thread_stack.as_ptr() as usize + STACK_SIZE;

    let stack_base = stack_top - args.len();
    let args_slice = unsafe {
        core::slice::from_raw_parts_mut(stack_base as *mut u8, args.len())
    };

    println!("{:?}", args);

    args_slice.copy_from_slice(args);

    let mut thread_context = context::Context::default();
    thread_context.sp = stack_base;
    thread_context.a0 = args_slice.as_ptr() as usize;
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

    let mut threads = THREADS.lock();
    let mut queue = QUEUE.lock();
    let id = threads.insert(thread);
    queue.push_back(id);

    return id;
}

pub fn delete_thread(id: usize) -> bool {
    let mut threads = THREADS.lock();
    if let Some(thread) = threads.get_mut(id) {
        thread.dead = true;
        return true;
    }
    return false;
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
    let mut threads = THREADS.lock();
    let mut queue = QUEUE.lock();

    let hart_info = unsafe {
        let hart_info_ptr = riscv::register::sscratch::read();
        &mut *(hart_info_ptr as *mut hart::HartInfo)
    };

    if let Some(current_thread_id) = hart_info.current_thread {
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
                    hart_info.current_thread = Some(new_thread_id);
                    return;
                }
            }
            None => {
                hart_info.current_thread = None;
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

pub fn get_current_thread() -> Option<usize> {
    let hart_info = unsafe {
        let hart_info_ptr = riscv::register::sscratch::read();
        &mut *(hart_info_ptr as *mut hart::HartInfo)
    };
    hart_info.current_thread
}
