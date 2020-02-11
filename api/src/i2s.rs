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
    pub bclk:  gpio_num_t,
    pub lrclk: gpio_num_t,
    pub din:   gpio_num_t,
    pub dout:  gpio_num_t,
}

impl Pins {
    pub fn new() -> Pins {
        Pins {
            bclk:  gpio_num_t::GPIO_NUM_5,
            lrclk: gpio_num_t::GPIO_NUM_25,
            din:   gpio_num_t::GPIO_NUM_26,
            dout:  gpio_num_t::GPIO_NUM_35,
        }
    }
}

use core::convert::From;

impl From<Pins> for i2s_pin_config_t {
    fn from(pins: Pins) -> Self {
        i2s_pin_config_t {
            bck_io_num:   pins.bclk  as i32,
            ws_io_num:    pins.lrclk as i32,
            data_in_num:  pins.din   as i32,
            data_out_num: pins.dout  as i32,
        }
    }
}

impl From<i2s_pin_config_t> for Pins {
    fn from(i2s_pin_config: i2s_pin_config_t) -> Self {
        Pins {
            bclk:  unsafe { core::mem::transmute::<i32, gpio_num_t>(i2s_pin_config.bck_io_num)   },
            lrclk: unsafe { core::mem::transmute::<i32, gpio_num_t>(i2s_pin_config.ws_io_num)    },
            din:   unsafe { core::mem::transmute::<i32, gpio_num_t>(i2s_pin_config.data_in_num)  },
            dout:  unsafe { core::mem::transmute::<i32, gpio_num_t>(i2s_pin_config.data_out_num) },
        }
    }
}
