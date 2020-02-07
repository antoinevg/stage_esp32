#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::fmt;

use cty;


// - modules ------------------------------------------------------------------

//pub mod errors;


// - esp-idf bindings ---------------------------------------------------------

pub mod bindings {
    pub mod std {
        pub mod os {
            pub mod raw {
                pub use cty::*;
            }
        }
    }
    include!("bindings.rs");
}


// - conversions --------------------------------------------------------------

use core::convert::From;

impl From<bindings::in_addr> for bindings::ip4_addr {
    fn from (item: bindings::in_addr) -> Self {
        bindings::ip4_addr { addr: item.s_addr }
    }
}

/*impl From<bindings::ip4_addr> for bindings::in_addr {
    fn from (item: bindings::ip4_addr) -> Self {
        bindings::in_addr { s_addr: item.addr }
    }
}*/

/*impl From<bindings::sockaddr_in> for bindings::sockaddr {
    fn from (item: bindings::sockaddr_in) -> Self {
        bindings::sockaddr {
            sa_len: item.sin_len,
            sa_family: item.sin_family,
            sa_data: [0; 14], // TODO
        }
    }
}*/


// - EspError -----------------------------------------------------------------
//
// Original is defined as: typedef int32_t esp_err_t;
//
// See: <esp-idf>/components/esp_common/include/esp_err.h

#[derive(Copy, Clone, Debug)]
pub struct EspError(pub bindings::esp_err_t);

pub trait AsResult<T, E> {
    fn as_result(self) -> Result<T, E>;
}
impl AsResult<(), EspError> for bindings::esp_err_t {
    fn as_result(self) -> Result<(), EspError> {
        if self == 0 {
            Ok(())
        } else {
            Err(EspError(self))
        }
    }
}


impl From<bindings::esp_err_t> for EspError {
    fn from(err: bindings::esp_err_t) -> Self {
        EspError(err)
    }
}

impl From<()> for EspError {
    fn from(_err: ()) -> Self {
        EspError(1)
    }
}


impl Into<Result<(), EspError>> for EspError {
    fn into(self) -> Result<(), EspError> {
        if self.0 == 0 { Ok(()) } else { Err(self) }
    }
}


impl fmt::Display for EspError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO {}", self)
    }
}


// TODO
/*
#define ESP_ERROR_CHECK(x) do {                                         \
        bindings::esp_err_t __err_rc = (x);                                       \
        if (__err_rc != ESP_OK) {                                       \
            _esp_error_check_failed(__err_rc, __FILE__, __LINE__,       \
                                    __ASSERT_FUNC, #x);                 \
        }                                                               \
    } while(0)
#endif
*/
//pub type esp_err_t = i32;
//pub fn esp_err_to_name(code: esp_err_t) -> *const ctypes::c_char;
//pub fn esp_err_to_name_r(code: esp_err_t, buf: *mut ctypes::c_char, buflen: usize) -> *const ctypes::c_char;


// - errno --------------------------------------------------------------------

// TODO
extern "C" {
    //#[no_mangle]
    //pub fn __errno() -> *const ctypes::c_int;
    //pub fn __errno() -> *mut ctypes::c_int;
    //pub fn strerror(arg1: ctypes::c_int) -> *mut ctypes::c_char; TODO
}

pub fn errno() -> cty::c_int {
    unsafe { *bindings::__errno() }
}



// - misc constants -----------------------------------------------------------

// actually CONFIG_FREERTOS_HZ which is 100 by default
pub const configTICK_RATE_HZ: bindings::TickType_t = bindings::configTICK_RATE_HZ;
pub const portTICK_PERIOD_MS: bindings::TickType_t = (1000 as bindings::TickType_t) / configTICK_RATE_HZ;
pub const portTICK_RATE_MS:   bindings::TickType_t = portTICK_PERIOD_MS;

pub const portMAX_DELAY: bindings::TickType_t = 0xffffffff;

pub const portMUX_INITIALIZER_UNLOCKED: bindings::portMUX_TYPE = bindings::portMUX_TYPE {
    owner: 0xb33f_ffff,
    count: 0,
};
