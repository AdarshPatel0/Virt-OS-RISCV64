use core::fmt::{self, Write};
use spin::mutex::Mutex;

struct Console;

static CONSOLE: Mutex<Console> = Mutex::new(Console); 

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            sbi::legacy::console_putchar(byte);
        }
        Ok(())
    }
}

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    CONSOLE.lock().write_fmt(args).unwrap();
}

macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print::print_args(format_args!($($arg)*));
    };
}

macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($fmt:expr) => {
        $crate::print::print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print::print!(
            concat!($fmt, "\n"),
            $($arg)*
        );
    };
}

pub(crate) use print;
pub(crate) use println;