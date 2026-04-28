use crate::libraries::console_utils::println;

pub fn system_shutdown(message: &str) -> ! {
    println!("{}",message);
    unsafe {
        core::arch::asm!(
            "
                li a7, 9
                ecall
            "
        )
    }
    loop {}
}
