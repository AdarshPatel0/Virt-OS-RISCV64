pub mod console_utils;

use sbi::system_reset::{ResetReason, ResetType};

pub fn shutdown() -> ! {
    match sbi::system_reset::system_reset(ResetType::Shutdown, ResetReason::NoReason) {
        Ok(_) => loop {},
        Err(_) => {
            unreachable!("Shutdown failed.")
        }
    }
}
