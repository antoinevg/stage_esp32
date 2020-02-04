use cty::{c_char, c_int, c_uint};

use esp_idf::{AsResult, EspError, portMAX_DELAY};
use esp_idf::bindings as idf;

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "wrap::i2s";

pub const PORT: idf::i2s_port_t = idf::i2s_port_t::I2S_NUM_0;

const USE_QUEUE: bool = false;
const QUEUE_SIZE: c_int = 128;
static mut QUEUE: Option<idf::QueueHandle_t> = None;


// - types --------------------------------------------------------------------

const QUEUE_TYPE_BASE: u8 = 0;


// - exports ------------------------------------------------------------------

pub unsafe fn init(fs: f32, block_size: usize) -> Result<(), EspError> {
    // configure i2s
    let i2s_config = idf::i2s_config_t {
        mode: idf::i2s_mode_t::I2S_MODE_MASTER
            | idf::i2s_mode_t::I2S_MODE_RX
            | idf::i2s_mode_t::I2S_MODE_TX
            | idf::i2s_mode_t::I2S_MODE_DAC_BUILT_IN
            | idf::i2s_mode_t::I2S_MODE_ADC_BUILT_IN,
        sample_rate: fs as c_int, //_scaled as c_int,
        bits_per_sample: idf::i2s_bits_per_sample_t::I2S_BITS_PER_SAMPLE_16BIT,
        channel_format: idf::i2s_channel_fmt_t::I2S_CHANNEL_FMT_RIGHT_LEFT,
        communication_format: idf::i2s_comm_format_t::I2S_COMM_FORMAT_I2S_MSB,
        intr_alloc_flags: idf::ESP_INTR_FLAG_LEVEL1 as i32,
        dma_buf_count: 4,
        dma_buf_len: block_size as i32,
        //use_apll: false,
        //fixed_mclk: 12_288_000,
        ..idf::i2s_config_t::default()
    };

    // install driver
    if USE_QUEUE {
        let handle: idf::QueueHandle_t = idf::xQueueGenericCreate(QUEUE_SIZE as c_uint,
                                                                  core::mem::size_of::<*const c_char> as c_uint,
                                                                  QUEUE_TYPE_BASE);
        QUEUE = Some(handle);
        idf::i2s_driver_install(PORT, &i2s_config, QUEUE_SIZE, QUEUE.unwrap());
    } else {
        idf::i2s_driver_install(PORT, &i2s_config, 0, core::ptr::null_mut());
    }

    // enable dac
    idf::i2s_set_pin(PORT, core::ptr::null()); // enables both internal DAC channels
    idf::i2s_set_dac_mode(idf::i2s_dac_mode_t::I2S_DAC_CHANNEL_BOTH_EN); // gpio 25, 26

    // enable adc
    idf::i2s_set_adc_mode(idf::adc_unit_t::ADC_UNIT_1, idf::adc1_channel_t::ADC1_CHANNEL_6); // gpio 34
    //idf::i2s_set_adc_mode(idf::adc_unit_t::ADC_UNIT_1, idf::adc1_channel_t::ADC1_CHANNEL_7); // gpio 35
    idf::i2s_adc_enable(PORT);

    // zero dma buffer
    idf::i2s_zero_dma_buffer(PORT);

    Ok(())
}
