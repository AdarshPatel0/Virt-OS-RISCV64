use crate::{ecall, threads};

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn trap_handler_entry() {
    core::arch::naked_asm!(
        r#"
        # Allocate Context on stack
        addi sp, sp, -264

        # -------------------------
        # Save registers (ABI order)
        # -------------------------

        sd ra,   0*8(sp)
        sd sp,   1*8(sp)
        sd gp,   2*8(sp)
        sd tp,   3*8(sp)

        sd t0,   4*8(sp)
        sd t1,   5*8(sp)
        sd t2,   6*8(sp)

        sd s0,   7*8(sp)
        sd s1,   8*8(sp)

        sd a0,   9*8(sp)
        sd a1,  10*8(sp)
        sd a2,  11*8(sp)
        sd a3,  12*8(sp)
        sd a4,  13*8(sp)
        sd a5,  14*8(sp)
        sd a6,  15*8(sp)
        sd a7,  16*8(sp)

        sd s2,  17*8(sp)
        sd s3,  18*8(sp)
        sd s4,  19*8(sp)
        sd s5,  20*8(sp)
        sd s6,  21*8(sp)
        sd s7,  22*8(sp)
        sd s8,  23*8(sp)
        sd s9,  24*8(sp)
        sd s10, 25*8(sp)
        sd s11, 26*8(sp)

        sd t3,  27*8(sp)
        sd t4,  28*8(sp)
        sd t5,  29*8(sp)
        sd t6,  30*8(sp)

        # -------------------------
        # Save CSRs
        # -------------------------
        csrr t0, sstatus
        sd   t0, 31*8(sp)

        csrr t0, sepc
        sd   t0, 32*8(sp)

        # -------------------------
        # Call Rust handler
        # -------------------------
        mv a0, sp
        call trap_handler

        # -------------------------
        # Restore CSRs
        # -------------------------
        ld t0, 31*8(sp)
        csrw sstatus, t0

        ld t0, 32*8(sp)
        csrw sepc, t0

        # -------------------------
        # Restore registers
        # -------------------------

        ld ra,   0*8(sp)
        ld sp,   1*8(sp)
        ld gp,   2*8(sp)
        ld tp,   3*8(sp)

        ld t0,   4*8(sp)
        ld t1,   5*8(sp)
        ld t2,   6*8(sp)

        ld s0,   7*8(sp)
        ld s1,   8*8(sp)

        ld a0,   9*8(sp)
        ld a1,  10*8(sp)
        ld a2,  11*8(sp)
        ld a3,  12*8(sp)
        ld a4,  13*8(sp)
        ld a5,  14*8(sp)
        ld a6,  15*8(sp)
        ld a7,  16*8(sp)

        ld s2,  17*8(sp)
        ld s3,  18*8(sp)
        ld s4,  19*8(sp)
        ld s5,  20*8(sp)
        ld s6,  21*8(sp)
        ld s7,  22*8(sp)
        ld s8,  23*8(sp)
        ld s9,  24*8(sp)
        ld s10, 25*8(sp)
        ld s11, 26*8(sp)

        ld t3,  27*8(sp)
        ld t4,  28*8(sp)
        ld t5,  29*8(sp)
        ld t6,  30*8(sp)

        # Restore original sp from context
        ld sp, 1*8(sp)

        # Free stack frame (restore original stack pointer already done above logically)
        # (No addi sp needed since sp was restored from context)

        sret
        "#
    );
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(context: &mut threads::Context) {
    let raw_trap = riscv::register::scause::read().cause();
    let trap: riscv::interrupt::Trap<riscv::interrupt::Interrupt, riscv::interrupt::Exception> = raw_trap.try_into().unwrap();
    match trap {
        riscv::interrupt::Trap::Interrupt(interrupt) => match interrupt {
            riscv::interrupt::Interrupt::SupervisorSoft => todo!(),
            riscv::interrupt::Interrupt::SupervisorTimer => {
                let time = riscv::register::time::read64();
                sbi::timer::set_timer(time + 1_000_000).unwrap();
            }
            riscv::interrupt::Interrupt::SupervisorExternal => todo!(),
        },
        riscv::interrupt::Trap::Exception(exception) => match exception {
            riscv::interrupt::Exception::InstructionMisaligned => todo!(),
            riscv::interrupt::Exception::InstructionFault => todo!(),
            riscv::interrupt::Exception::IllegalInstruction => todo!(),
            riscv::interrupt::Exception::Breakpoint => todo!(),
            riscv::interrupt::Exception::LoadMisaligned => todo!(),
            riscv::interrupt::Exception::LoadFault => todo!(),
            riscv::interrupt::Exception::StoreMisaligned => todo!(),
            riscv::interrupt::Exception::StoreFault => todo!(),
            riscv::interrupt::Exception::UserEnvCall => {
                ecall::call(context);
            }
            riscv::interrupt::Exception::SupervisorEnvCall => {
                ecall::call(context);
            }
            riscv::interrupt::Exception::InstructionPageFault => todo!(),
            riscv::interrupt::Exception::LoadPageFault => todo!(),
            riscv::interrupt::Exception::StorePageFault => todo!(),
        },
    }
    return;
}
