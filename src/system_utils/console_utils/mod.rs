use core::fmt::{self, Write};

use crate::ecall;

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            sbi::legacy::console_putchar(byte);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    Console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::system_utils::console_utils::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(
            concat!($fmt, "\n"),
            $($arg)*
        );
    };
}

#[allow(dead_code)]
pub fn get_char() -> u8 {
    let call_code = ecall::get_code(ecall::Ecall::GetChar);
    let mut a0: usize = 0;
    unsafe {
        core::arch::asm!(
            "
            mv  a7, {0}
            ecall
            mv  {1}, a0
            ",
            in(reg) call_code,
            out(reg) a0,
        );
    }
    return a0 as u8;
}

#[allow(dead_code)]
pub fn print_string(string: &str) {
    let call_code = ecall::get_code(ecall::Ecall::PrintString);
    let string_start = string.as_ptr() as usize;
    let string_length = string.len();
    unsafe {
        core::arch::asm!(
            "
            mv  a7, {0}
            mv  a0, {1}
            mv  a1, {2}
            ecall
            ",
            in(reg) call_code,
            in(reg) string_start,
            in(reg) string_length,
        );
    }
}

#[allow(dead_code)]
pub fn print_char(character: u8) {
    let call_code = ecall::get_code(ecall::Ecall::PrintChar);
    let c = character as usize;
    unsafe {
        core::arch::asm!(
            "
            mv  a7, {0}
            mv  a0, {1}
            ecall
            ",
            in(reg) call_code,
            in(reg) c,
        );
    }
}
