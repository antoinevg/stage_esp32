use esp_idf::{AsResult, EspError, portMAX_DELAY, portTICK_RATE_MS};
use esp_idf::bindings::{
    gpio_num_t,
    spi_host_device_t,
};
use esp_idf::bindings as idf;

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver::sh1106::i2c";


// - types --------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Pins {
    pub csel: gpio_num_t,
    pub sclk: gpio_num_t,
    pub mosi: gpio_num_t,
    pub dc: gpio_num_t,
}

impl Pins {
    pub fn new() -> Pins {
        Pins {
            csel:  gpio_num_t::GPIO_NUM_5,
            sclk:  gpio_num_t::GPIO_NUM_18,
            mosi:  gpio_num_t::GPIO_NUM_23,
            dc:    gpio_num_t::GPIO_NUM_19,
        }
    }
}


// - initialization -----------------------------------------------------------

pub unsafe fn init(port: spi_host_device_t, pins: Pins) -> Result<(), EspError> {
    log!(TAG, "configure spi pins for display peripheral: {:?}", pins);

    let config = gpio_config_t {
        pin_bit_mask: (0x1 << (pins.cs as u32)) | (0x1 << (pins.miso as u32)),
        mode: gpio_mode_t::GPIO_MODE_OUTPUT,
        pull_up_en: gpio_pullup_t::GPIO_PULLUP_DISABLE,
        pull_down_en: gpio_pulldown_t::GPIO_PULLDOWN_DISABLE,
        intr_type: gpio_int_type_t::GPIO_INTR_DISABLE,
    };

    Ok(())
}


pub unsafe fn configure(reset: gpio_num_t, port: spi_host_device_t, address: u8) -> Result<(), EspError> {
    log!(TAG, "configuring sh1106 oled display at address: 0x{:x}", address);

    Ok(())
}
