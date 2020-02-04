use core::fmt;
use core::fmt::Write;

use cty::c_char;

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError};


// - ffi imports --------------------------------------------------------------

extern "C" {
    fn write(fd: i32, data: *const u8, size: usize) -> isize;
}


// - implementation -----------------------------------------------------------

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            let buffer = s.as_bytes();
            let mut offset = 0;
            loop {
                let count = write(0, buffer[offset..].as_ptr(), buffer.len() - offset);
                if count < 0 {
                    return Err(()).map_err(|_| fmt::Error);
                }
                offset += count as usize;
                if offset == buffer.len() {
                    return Ok(())
                }
            }
        }
    }
}

pub fn print_fmt(args: fmt::Arguments) {
    let mut stdout = Stdout{};
    let ret = stdout.write_fmt(args).unwrap();
    ret
}

pub fn println_fmt(args: fmt::Arguments) {
    let mut stdout = Stdout{};
    let ret = stdout.write_fmt(args).unwrap();
    unsafe { write(0, "\n".as_bytes().as_ptr(), 1); }
    ret
}

pub fn println_fmt_tag(tag: &str, args: fmt::Arguments) {
    let mut stdout = Stdout{};
    let ret = stdout.write_fmt(format_args!("[{}] ", tag)).unwrap();
    let ret = stdout.write_fmt(args).unwrap();
    unsafe { write(0, "\n".as_bytes().as_ptr(), 1); }
    ret
}


// - interface ----------------------------------------------------------------

pub fn esp_log(tag: &str, message: &str) {
    unsafe {
        idf::esp_log_write(idf::esp_log_level_t::ESP_LOG_INFO,
                           tag.as_ptr() as *const c_char,
                           message.as_ptr() as *const c_char);
    }
}
