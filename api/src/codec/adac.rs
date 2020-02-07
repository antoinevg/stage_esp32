use cty::{c_float, c_void};

use esp_idf::{AsResult, EspError, portMAX_DELAY};
use esp_idf::bindings as idf;

use crate::audio::{Buffer, Config, Interface, OpaqueInterface};
use crate::codec::Codec;
use crate::i2s;
use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::codec::adac";


// - driver -------------------------------------------------------------------

pub struct Driver {
    dma_buffer_ptr: *mut u8,
}


unsafe impl Codec for Driver {
    fn new() -> Driver {
        Driver {
            dma_buffer_ptr: core::ptr::null_mut()
        }
    }

    fn init(&mut self, config: &Config) -> Result<(), EspError> {

        log!(TAG, "initialize i2s driver with fs: {}", config.fs);
        unsafe { i2s::init(config.fs, config.block_size)?; }

        let buffer_size = config.block_size * config.word_size;
        self.dma_buffer_ptr = unsafe {
            idf::calloc(buffer_size as u32,
                        core::mem::size_of::<u8>() as u32) as *mut u8
        };
        if self.dma_buffer_ptr == core::ptr::null_mut() {
            return (idf::ESP_ERR_NO_MEM as idf::esp_err_t).as_result();
        }
        log!(TAG, "allocated memory for dma buffer: {} bytes", buffer_size);

        Ok(())
    }

    fn read(&self, config: &Config, callback_buffer: &mut [f32]) -> Result<(), EspError> {
        let Config { num_channels, word_size, block_size, .. } = config;
        let buffer_size = block_size * word_size;
        let num_frames  = block_size / num_channels;

        let dma_buffer = unsafe {
            core::slice::from_raw_parts_mut(self.dma_buffer_ptr, buffer_size)
        };

        // read audio data from i2s
        let mut bytes_read = 0;
        unsafe {
            idf::i2s_read(i2s::PORT,
                          dma_buffer.as_ptr() as *mut core::ffi::c_void,
                          buffer_size,
                          &mut bytes_read,
                          portMAX_DELAY).as_result()?;
        }
        if bytes_read != buffer_size {
            log!(TAG, "read mismatch buffer_size:{} != bytes_read:{}", buffer_size, bytes_read);
            return (idf::ESP_ERR_INVALID_SIZE as idf::esp_err_t).as_result();
        }

        // convert audio data from u12 to f32
        for n in 0..num_frames {
            let index_u8 = n * num_channels * word_size;

            let right_lo: u8 = dma_buffer[index_u8+0];
            let right_hi: u8 = dma_buffer[index_u8+1];
            let left_lo:  u8 = dma_buffer[index_u8+2];
            let left_hi:  u8 = dma_buffer[index_u8+3];

            let right_u16: u16 = (((right_hi as u16) & 0xf) << 8) | (right_lo as u16);
            let left_u16:  u16 = (((left_hi  as u16) & 0xf) << 8) | (left_lo as u16);

            let right_f32 = (((right_u16 as f32) / 4095.) * 2.) - 1.;
            let left_f32  = (((left_u16  as f32) / 4095.) * 2.) - 1.;

            let right_f32 = if right_f32 > 1. { 1. } else if right_f32 < -1. { -1. } else { right_f32 };
            let left_f32  = if left_f32  > 1. { 1. } else if left_f32  < -1. { -1. } else { left_f32  };

            let index_f32 = n * num_channels;
            callback_buffer[index_f32+0] = right_f32;
            callback_buffer[index_f32+1] = left_f32;
        }

        Ok(())
    }

    fn write(&self, config: &Config, buffer: &Buffer) -> Result<(), EspError> {
        let Config { num_channels, word_size, block_size, .. } = config;
        let buffer_size = block_size * word_size;
        let num_frames  = block_size / num_channels;

        let dma_buffer = unsafe {
            core::slice::from_raw_parts_mut(self.dma_buffer_ptr, buffer_size)
        };

        // convert audio data from f32 to u8
        for n in 0..num_frames {
            let index_f32 = n * num_channels;
            let right_f32 = buffer[index_f32+0];
            let left_f32  = buffer[index_f32+1];

            let right_f32 = if right_f32 > 1. { 1. } else if right_f32 < -1. { -1. } else { right_f32 };
            let left_f32  = if left_f32  > 1. { 1. } else if left_f32  < -1. { -1. } else { left_f32  };

            let right_u8: u8 = ((((right_f32 + 1.) * 0.5) * 255.0) as u32) as u8;
            let left_u8:  u8 = ((((left_f32  + 1.) * 0.5) * 255.0) as u32) as u8;

            let index_u8 = n * num_channels * word_size;
            dma_buffer[index_u8+0] = 0;
            dma_buffer[index_u8+1] = right_u8;
            dma_buffer[index_u8+2] = 0;
            dma_buffer[index_u8+3] = left_u8;
        }

        // write audio data to i2s
        let mut bytes_written = 0;
        unsafe {
            idf::i2s_write(i2s::PORT,
                           dma_buffer.as_ptr() as *const core::ffi::c_void,
                           buffer_size,
                           &mut bytes_written,
                           portMAX_DELAY).as_result()?;
        }
        if bytes_written != buffer_size {
            log!(TAG, "write mismatch buffer_size:{} != bytes_written:{}", buffer_size, bytes_written);
            return (idf::ESP_ERR_INVALID_SIZE as idf::esp_err_t).as_result();
        }

        Ok(())
    }

    fn start_c(&self, config: &Config,
               opaque_interface_ptr: *const OpaqueInterface) -> Result<(), EspError> {
        unsafe {
            C_codec_adac_start(opaque_interface_ptr,
                               config.fs,
                               config.num_channels,
                               config.word_size,
                               config.block_size).as_result()
        }
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
