pub mod context;

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec;
use slab::Slab;

use crate::{print, println};

const STACK_SIZE: usize = 4096;

#[derive(PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Dead,
}
pub struct Thread {
    pub context: context::Context,
    pub stack: Box<[u8]>,
    pub state: ThreadState,
}

pub static mut THREADS: Slab<Thread> = Slab::new();
static mut RUN_QUEUE: VecDeque<usize> = VecDeque::new();
static mut CURRENT: usize = 0;

pub fn create_thread(entry: usize, user_mode: bool) -> usize {
    let stack = vec![0u8; STACK_SIZE].into_boxed_slice();
    let stack_top = stack.as_ptr() as usize + STACK_SIZE;

    let mut thread_context = context::Context::default();
    thread_context.sp = stack_top;
    thread_context.sepc = entry;

    // --- Logic for User Mode ---
    let mut sstatus_val = riscv::register::sstatus::read();

    if user_mode {
        // Set SPP (Supervisor Previous Privilege) to User (0)
        sstatus_val.set_spp(riscv::register::sstatus::SPP::User);
    } else {
        // Set SPP to Supervisor (1)
        sstatus_val.set_spp(riscv::register::sstatus::SPP::Supervisor);
    }

    // Usually, you want to ensure interrupts are enabled when the thread starts.
    // SPIE (Supervisor Previous Interrupt Enable) will be copied to SIE on sret.
    sstatus_val.set_spie(true);

    thread_context.sstatus = sstatus_val.bits();

    let thread = Thread {
        context: thread_context,
        stack,
        state: ThreadState::Ready,
    };

    unsafe {
        let threads = &raw mut THREADS;
        let id = (*threads).insert(thread);

        let run_queue = &raw mut RUN_QUEUE;
        (*run_queue).push_back(id);

        id
    }
}

pub fn delete_thread(id: usize) -> bool {
    unsafe {
        let threads = &raw mut THREADS;
        match (*threads).get_mut(id) {
            Some(thread) => {
                thread.state = ThreadState::Dead;
                true
            }
            None => false,
        }
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wait_for_interrupt() {
    core::arch::naked_asm!(
        r#"
        wfi 
        "#
    );
}

pub fn schedule(current_context: &mut context::Context) {
    unsafe {
        let threads = &mut *(&raw mut THREADS);
        let run_queue = &mut *(&raw mut RUN_QUEUE);

        // 1. Save current context back to the Slab if the current thread still exists
        if let Some(current_thread) = threads.get_mut(CURRENT) {
            // Copy the registers from the trap frame/context into our thread storage
            current_thread.context = core::ptr::read(current_context);
            
            // If it was running, put it back to Ready so it can be scheduled later
            if current_thread.state == ThreadState::Running {
                current_thread.state = ThreadState::Ready;
            }
        }

        // 2. Add the current thread back to the end of the queue to maintain Round Robin
        // (Only if it's not Dead; though Dead threads are usually handled during popping)
        run_queue.push_back(CURRENT);

        // 3. Find the next thread to run
        loop {
            match run_queue.pop_front() {
                Some(next_id) => {
                    if let Some(thread) = threads.get_mut(next_id) {
                        if thread.state == ThreadState::Dead {
                            // Found a Dead thread: Remove from Slab (frees Box memory)
                            threads.remove(next_id);
                            continue; // Keep looking for a live thread
                        }

                        // Found a valid thread to run
                        println!("{}", next_id);
                        CURRENT = next_id;
                        thread.state = ThreadState::Running;
                        
                        // Copy thread context into current_context for the asm wrapper to restore
                        core::ptr::write(current_context, thread.context);
                        return;
                    } else {
                        // ID was in queue but not in slab, skip
                        continue;
                    }
                }
                None => {
                    // 4. No threads left in the queue: Idle logic
                    // Set sstatus to Supervisor mode (assuming MPP bits in RISC-V)
                    // Clear the SPP bit (bit 8) to return to Supervisor (0) instead of User (1)
                    // and ensure SPIE (bit 5) is handled if you want interrupts enabled on return.
                    current_context.sstatus &= !(1 << 8); 
                    current_context.sepc = wait_for_interrupt as usize;
                    return;
                }
            }
        }
    }
}