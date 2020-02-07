#![no_std]

#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(lang_items)]

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]


// - macros -------------------------------------------------------------------

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (logger::print_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => (logger::println_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log {
    ($tag: ident, $($arg:tt)*) => (logger::println_fmt_tag($tag, format_args!($($arg)*)));
}


// - modules ------------------------------------------------------------------

pub mod allocators;
pub mod audio;
pub mod blinky;
pub mod driver;
pub mod i2s;
pub mod ledc;
pub mod logger;
pub mod lwip;
pub mod nvs;
pub mod wavetable;
pub mod wifi;


// - panic handler ------------------------------------------------------------

use core::intrinsics;
use core::panic::PanicInfo;

#[lang = "panic_impl"]
extern fn rust_begin_panic(_info: &PanicInfo) -> ! {
    unsafe { intrinsics::abort() }
}


// - tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
