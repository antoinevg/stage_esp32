use cty::{c_float, c_void};

use esp_idf::{AsResult, EspError, portMAX_DELAY};
use esp_idf::bindings as idf;

use crate::audio::{Buffer, Config, Interface, OpaqueInterface};
use crate::driver::Codec;
use crate::logger;
use crate::i2s::{Pins};


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver::adac";


// - driver -------------------------------------------------------------------

pub struct Driver {
    dma_buffer_ptr: *mut u8,
}


unsafe impl Codec for Driver {
    fn new() -> Driver {
        Driver {
            dma_buffer_ptr: core::ptr::null_mut(),
        }
    }

    fn init(&mut self, config: &Config) -> Result<(), EspError> {
        let port = idf::i2s_port_t::I2S_NUM_0;

        log!(TAG, "initialize audio subsystem with fs:{} block_length:{}", config.fs, config.block_length);

        // allocate memory for dma buffer
        let buffer_size = config.block_length * config.word_size;
        self.dma_buffer_ptr = unsafe {
            idf::calloc(buffer_size as u32,
                        core::mem::size_of::<u8>() as u32) as *mut u8
        };
        if self.dma_buffer_ptr == core::ptr::null_mut() {
            return (idf::ESP_ERR_NO_MEM as idf::esp_err_t).as_result();
        }
        log!(TAG, "allocated memory for dma buffer: {} bytes", buffer_size);

        // initialize i2s peripheral
        log!(TAG, "initialize i2s peripheral");
        unsafe { i2s::init(port, config.fs, config.block_length)?; }

        Ok(())
    }

    fn read(&self, config: &Config, callback_buffer: &mut [f32]) -> Result<(), EspError> {
        let Config { num_channels, word_size, block_length, .. } = config;
        let buffer_size = block_length * word_size;
        let num_frames  = block_length / num_channels;

        let dma_buffer = unsafe {
            core::slice::from_raw_parts_mut(self.dma_buffer_ptr, buffer_size)
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

        // convert audio data from u12 to f32
        for n in 0..num_frames {
            let index_u8 = n * num_channels * word_size;

            let right_lo: u8 = dma_buffer[index_u8+0];
            let right_hi: u8 = dma_buffer[index_u8+1];
            let left_lo:  u8 = dma_buffer[index_u8+2];
            let left_hi:  u8 = dma_buffer[index_u8+3];

            // TODO use from_le_bytes - https://doc.rust-lang.org/std/primitive.u32.html#method.from_le_bytes
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
        let Config { num_channels, word_size, block_length, .. } = config;
        let buffer_size = block_length * word_size;
        let num_frames  = block_length / num_channels;

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
        unsafe {
            C_api_driver_adac_start(opaque_interface_ptr,
                                    config.fs,
                                    config.num_channels,
                                    config.word_size,
                                    config.block_length).as_result()
        }
    }
}


// - i2s ----------------------------------------------------------------------

pub mod i2s {
    use cty::{c_char, c_int, c_uint};

    use esp_idf::{AsResult, EspError, portMAX_DELAY};
    use esp_idf::bindings as idf;
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

    use crate::i2s::{Pins};

    const USE_QUEUE: bool = false;
    const QUEUE_SIZE: c_int = 128;
    static mut QUEUE: Option<idf::QueueHandle_t> = None;
    const QUEUE_TYPE_BASE: u8 = 0;

    pub unsafe fn init(port: i2s_port_t, fs: f32, block_length: usize) -> Result<(), EspError> {
        // configure i2s
        let i2s_config = i2s_config_t {
            mode: i2s_mode_t::I2S_MODE_MASTER
                | i2s_mode_t::I2S_MODE_RX
                | i2s_mode_t::I2S_MODE_TX
                | i2s_mode_t::I2S_MODE_DAC_BUILT_IN
                | i2s_mode_t::I2S_MODE_ADC_BUILT_IN,
            sample_rate: fs as c_int, //_scaled as c_int,
            bits_per_sample: i2s_bits_per_sample_t::I2S_BITS_PER_SAMPLE_16BIT,
            channel_format: i2s_channel_fmt_t::I2S_CHANNEL_FMT_RIGHT_LEFT,
            communication_format: i2s_comm_format_t::I2S_COMM_FORMAT_I2S_MSB,
            intr_alloc_flags: ESP_INTR_FLAG_LEVEL1 as i32,
            dma_buf_count: 4,
            dma_buf_len: block_length as i32,
            use_apll: false,
            //fixed_mclk: 12_288_000,
            ..i2s_config_t::default()
        };

        // install driver
        if USE_QUEUE {
            let handle: idf::QueueHandle_t = idf::xQueueGenericCreate(QUEUE_SIZE as c_uint,
                                                                      core::mem::size_of::<*const c_char> as c_uint,
                                                                      QUEUE_TYPE_BASE);
            QUEUE = Some(handle);
            i2s_driver_install(port, &i2s_config, QUEUE_SIZE, QUEUE.unwrap()).as_result()?;
        } else {
            i2s_driver_install(port, &i2s_config, 0, core::ptr::null_mut()).as_result()?;
        }

        // enable dac
        i2s_set_pin(port, core::ptr::null()).as_result()?; // enables both internal DAC channels
        idf::i2s_set_dac_mode(idf::i2s_dac_mode_t::I2S_DAC_CHANNEL_BOTH_EN).as_result()?; // gpio 25, 26

        // enable adc
        idf::i2s_set_adc_mode(idf::adc_unit_t::ADC_UNIT_1, idf::adc1_channel_t::ADC1_CHANNEL_6).as_result()?; // gpio 34
        //idf::i2s_set_adc_mode(idf::adc_unit_t::ADC_UNIT_1, idf::adc1_channel_t::ADC1_CHANNEL_7).as_result()?; // gpio 35
        idf::i2s_adc_enable(port).as_result()?;

        // zero dma buffer
        i2s_zero_dma_buffer(port).as_result()?;

        Ok(())
    }

}


// - ffi imports --------------------------------------------------------------

extern "C" {
    pub fn C_api_driver_adac_start(opaque_interface_ptr: *const OpaqueInterface,
                                   fs: f32,
                                   num_channels: usize,
                                   word_size: usize,
                                   block_length: usize) -> idf::esp_err_t;
}


// - ffi exports --------------------------------------------------------------

#[no_mangle]
extern "C" fn RUST_api_driver_adac_callback(opaque_interface_ptr: *const OpaqueInterface,
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

    if buffer_size != config.block_length {
        panic!("api::driver::adac callback buffer size does not match interface block_length");
    }
    let buffer = unsafe {
        core::slice::from_raw_parts_mut(buffer_ptr, buffer_size)
    };

    closure(fs, num_channels, buffer);
}
