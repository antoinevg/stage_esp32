use cty::{c_float, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY};

use crate::audio;
use crate::audio::{Buffer, Config, Interface, OpaqueInterface};
use crate::codec::Codec;
use crate::i2s;
use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::codec::adac";


// - driver -------------------------------------------------------------------

pub struct Driver;


impl Codec for Driver {
    fn new() -> Driver {
        Driver { }
    }

    fn init(&self, config: &Config) -> Result<(), EspError> {
        log!(TAG, "initializing i2s driver with fs: {}", config.fs);
        unsafe { i2s::init(config.fs, config.block_size) }
    }

    /*fn read(&self, &mut Buffer) {
    }

    fn write(&self, &mut Buffer) {
    }*/

    fn start_c(&self, config: &Config,
               opaque_interface_ptr: *const OpaqueInterface) -> Result<(), EspError> {
        unsafe {
            C_codec_adac_start(opaque_interface_ptr,
                               config.fs,
                               config.num_channels,
                               config.word_size,
                               config.block_size).as_result()?;
        }

        Ok(())
    }
}


// - ffi imports --------------------------------------------------------------

extern "C" {
    pub fn C_codec_adac_start(opaque_interface_ptr: *const OpaqueInterface,
                              fs: f32,
                              num_channels: usize,
                              word_size: usize,
                              block_size: usize) -> idf::esp_err_t;
}


// - ffi exports --------------------------------------------------------------

#[no_mangle]
extern "C" fn RUST_codec_adac_callback(opaque_interface_ptr: *const OpaqueInterface,
                                       fs: f32,
                                       num_channels: usize,
                                       buffer_ptr: *mut c_float,
                                       buffer_size: usize) {
    let interface_ptr = unsafe {
        core::mem::transmute::<*const OpaqueInterface,
                               *mut Interface<Driver>>(opaque_interface_ptr)
    };
    let config = unsafe { &(*interface_ptr).config };
    let closure = unsafe { &mut (*interface_ptr).closure };

    if buffer_size != config.block_size {
        panic!("api::codec::adac callback buffer size does not match interface block_size");
    }
    let buffer = unsafe {
        core::slice::from_raw_parts_mut(buffer_ptr, buffer_size)
    };

    closure(fs, num_channels, buffer);
}
