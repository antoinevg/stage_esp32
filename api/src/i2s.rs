use cty::{c_char, c_int, c_uint};

use esp_idf::{AsResult, EspError, portMAX_DELAY};
use esp_idf::bindings as idf;
use esp_idf::bindings::{
    gpio_num_t,
    i2s_port_t,
    i2s_config_t,
    i2s_pin_config_t,
};

use crate::logger;


// - types --------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Pins {
    pub bclk: u8,
    pub lrclk: u8,
    pub din: u8,
    pub dout: u8,
}

impl Pins {
    pub fn new() -> Pins {
        Pins {
            bclk:  gpio_num_t::GPIO_NUM_5  as u8,
            lrclk: gpio_num_t::GPIO_NUM_25 as u8,
            din:   gpio_num_t::GPIO_NUM_26 as u8,
            dout:  gpio_num_t::GPIO_NUM_35 as u8,
        }
    }
}

use core::convert::From;

impl From<Pins> for i2s_pin_config_t {
    fn from(pins: Pins) -> Self {
        i2s_pin_config_t {
            bck_io_num: pins.bclk as i32,
            ws_io_num: pins.lrclk as i32,
            data_in_num: pins.din as i32,
            data_out_num: pins.dout as i32,
        }
    }
}

impl From<i2s_pin_config_t> for Pins {
    fn from(i2s_pin_config: i2s_pin_config_t) -> Self {
        Pins {
            bclk: i2s_pin_config.bck_io_num as u8,
            lrclk: i2s_pin_config.ws_io_num as u8,
            din: i2s_pin_config.data_in_num as u8,
            dout: i2s_pin_config.data_out_num as u8,
        }
    }
}
