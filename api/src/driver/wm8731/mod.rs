use cty::{c_float};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY};

use crate::audio::{Buffer, Config, Interface, OpaqueInterface};
use crate::driver::Codec;
use crate::logger;

// - modules ------------------------------------------------------------------

pub mod i2c;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver::wm8731";


// - driver -------------------------------------------------------------------

pub struct Driver {
    pub i2c_pins: crate::i2c::Pins,
    pub i2s_pins: crate::i2s::Pins,
    dma_buffer_ptr: *mut i16,
}


unsafe impl Codec for Driver {
    fn new() -> Driver {
        Driver {
            i2c_pins: crate::i2c::Pins::new(),
            i2s_pins: crate::i2s::Pins::new(),
            dma_buffer_ptr: core::ptr::null_mut(),
        }
    }

    fn init(&mut self, config: &Config) -> Result<(), EspError> {
        let i2s_port = idf::i2s_port_t::I2S_NUM_0;
        let i2c_port = idf::i2c_port_t::I2C_NUM_0;

        log!(TAG, "initialize audio subsystem with fs:{} block_length:{}", config.fs, config.block_length);

        // allocate memory for dma buffer
        let buffer_size = config.block_length * config.word_size;
        self.dma_buffer_ptr = unsafe {
            idf::calloc(config.block_length as u32,
                        core::mem::size_of::<u16>() as u32) as *mut i16
        };
        if self.dma_buffer_ptr == core::ptr::null_mut() {
            return (idf::ESP_ERR_NO_MEM as idf::esp_err_t).as_result();
        }
        log!(TAG, "allocated memory for dma buffer: {} bytes", buffer_size);

        // initialize i2c peripheral
        log!(TAG, "initialize i2c peripheral");
        unsafe { i2c::init(i2c_port, self.i2c_pins)?; }

        // configure codec over i2c
        log!(TAG, "configure codec over i2c");
        let codec_i2c_address = 0x1a; // or 0x1b if CSB is high
        unsafe { i2c::configure(i2c_port, codec_i2c_address)?; }

        unsafe { idf::ets_delay_us(1000); } // give codec a few cycles to settle

        // initialize i2s peripheral
        log!(TAG, "initialize i2s peripheral");
        unsafe { i2s::init(i2s_port, self.i2s_pins, config)?; }

        unsafe { idf::ets_delay_us(1000); } // give i2s a few cycles to settle

        Ok(())
    }

    fn read(&self, config: &Config, callback_buffer: &mut [f32]) -> Result<(), EspError> {
        let Config { num_channels, word_size, block_length, .. } = config;
        let buffer_size = block_length * word_size;
        let num_frames  = block_length / num_channels;

        let dma_buffer = unsafe {
            core::slice::from_raw_parts_mut(self.dma_buffer_ptr, *block_length)
        };

        // read audio data from i2s
        let mut bytes_read = 0;
        unsafe {
            idf::i2s_read(idf::i2s_port_t::I2S_NUM_0,
                          self.dma_buffer_ptr as *mut core::ffi::c_void,
                          buffer_size,
                          &mut bytes_read,
                          portMAX_DELAY).as_result()?;
        }
        if bytes_read != buffer_size {
            log!(TAG, "read mismatch buffer_size:{} != bytes_read:{}", buffer_size, bytes_read);
            return (idf::ESP_ERR_INVALID_SIZE as idf::esp_err_t).as_result();
        }

        // convert audio data from i16 to f32
        for n in 0..num_frames {
            let index_i16 = n * num_channels;
            let right_i16: i16 = dma_buffer[index_i16+0];
            let left_i16:  i16 = dma_buffer[index_i16+1];

            let right_f32 = (right_i16 as f32) / 32768.;
            let left_f32  = (left_i16  as f32) / 32768.;

            let right_f32 = if right_f32 > 1. { 1. } else if right_f32 < -1. { -1. } else { right_f32 };
            let left_f32  = if left_f32  > 1. { 1. } else if left_f32  < -1. { -1. } else { left_f32  };

            let index_f32 = n * num_channels;
            callback_buffer[index_f32+0] = right_f32;
            callback_buffer[index_f32+1] = left_f32;
        }

        Ok(())
    }

    fn write(&self, config: &Config, callback_buffer: &[f32]) -> Result<(), EspError> {
        let Config { num_channels, word_size, block_length, .. } = config;
        let buffer_size = block_length * word_size;
        let num_frames  = block_length / num_channels;

        let dma_buffer = unsafe {
            core::slice::from_raw_parts_mut(self.dma_buffer_ptr, buffer_size)
        };

        // convert audio data from f32 to i16
        for n in 0..num_frames {
            let index_f32 = n * num_channels;
            let right_f32 = callback_buffer[index_f32+0];
            let left_f32  = callback_buffer[index_f32+1];

            let right_f32 = if right_f32 > 1. { 1. } else if right_f32 < -1. { -1. } else { right_f32 };
            let left_f32  = if left_f32  > 1. { 1. } else if left_f32  < -1. { -1. } else { left_f32  };

            let right_i16: i16 = (right_f32 * 32767.) as i16;
            let left_i16:  i16 = (left_f32  * 32767.) as i16;

            let index_i16 = n * num_channels;
            dma_buffer[index_i16+0] = right_i16;
            dma_buffer[index_i16+1] = left_i16;
        }

        // write audio data to i2s
        let mut bytes_written = 0;
        unsafe {
            idf::i2s_write(idf::i2s_port_t::I2S_NUM_0,
                           self.dma_buffer_ptr as *const core::ffi::c_void,
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
        // not supported
        Ok(())
    }
}


// - i2s initialization -------------------------------------------------------

pub mod i2s {
    use cty::{c_int};

    use esp_idf::{AsResult, EspError, portMAX_DELAY};
    use esp_idf::bindings::{
        ESP_INTR_FLAG_LEVEL1
    };
    use esp_idf::bindings::{
        gpio_num_t,
        i2s_port_t,
        i2s_config_t,
        i2s_mode_t,
        i2s_bits_per_sample_t,
        i2s_channel_fmt_t,
        i2s_comm_format_t,
        i2s_pin_config_t,
    };
    use esp_idf::bindings::{
        i2s_driver_install,
        i2s_set_pin,
        i2s_zero_dma_buffer,
        i2s_read,
        i2s_write,
    };

    use crate::audio;
    use crate::i2s::{Pins};
    use crate::logger;

    const TAG: &str = "api::driver::wm8731::i2s";

    pub unsafe fn init(port: i2s_port_t, pins: Pins, config: &audio::Config) -> Result<(), EspError> {
        // initialize i2s driver
        log!(TAG, "initialize i2s peripheral");
        let i2s_config = i2s_config_t {
            mode: i2s_mode_t::I2S_MODE_MASTER
                | i2s_mode_t::I2S_MODE_RX
                | i2s_mode_t::I2S_MODE_TX,
            sample_rate: config.fs as c_int,
            bits_per_sample: i2s_bits_per_sample_t::I2S_BITS_PER_SAMPLE_16BIT,
            channel_format: i2s_channel_fmt_t::I2S_CHANNEL_FMT_RIGHT_LEFT,
            communication_format: i2s_comm_format_t::I2S_COMM_FORMAT_I2S
                                | i2s_comm_format_t::I2S_COMM_FORMAT_I2S_MSB,
            intr_alloc_flags: ESP_INTR_FLAG_LEVEL1 as i32,
            dma_buf_count: 4,
            dma_buf_len: config.block_length as i32,
            use_apll: false,
            ..i2s_config_t::default()
        };
        i2s_driver_install(port, &i2s_config, 0, core::ptr::null_mut()).as_result()?;

        // configure pins for i2s peripheral
        log!(TAG, "configure pins for i2s peripheral: {:?}", pins);
        i2s_set_pin(port, &pins.into()).as_result()?;

        // zero dma buffer
        i2s_zero_dma_buffer(port).as_result()?;

        Ok(())
    }

}
