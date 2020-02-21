use esp_idf::{AsResult, EspError};
use esp_idf::bindings::{
    gpio_config_t,
    gpio_mode_t,
    gpio_num_t,
    gpio_pulldown_t,
    gpio_pullup_t,
    gpio_int_type_t,
};
use esp_idf::bindings::{
    gpio_config,
    gpio_set_level,
};

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::blinky";


// - registers ----------------------------------------------------------------

// https://github.com/espressif/esp-idf/blob/master/components/soc/esp32/include/soc/gpio_reg.h
mod gpio_reg {
    pub const GPIO_ENABLE_W1TS_REG: u32 = 0x3FF44024;
    pub const GPIO_OUT_W1TS_REG: u32 = 0x3FF44008;
    pub const GPIO_OUT_W1TC_REG : u32 = 0x3FF4400C;
    pub const GPIO_FUNCX_OUT_BASE: u32 = 0x3FF44530;
}


// - implementation -----------------------------------------------------------

pub fn configure_pin_as_output(gpio: gpio_num_t) -> Result<(), EspError> {
    log!(TAG, "configure pin for blinky: {:?}", gpio);
    let config = gpio_config_t {
        pin_bit_mask:  0x1 << (gpio as u32),
        mode: gpio_mode_t::GPIO_MODE_OUTPUT,
        pull_up_en: gpio_pullup_t::GPIO_PULLUP_DISABLE,
        pull_down_en: gpio_pulldown_t::GPIO_PULLDOWN_DISABLE,
        intr_type: gpio_int_type_t::GPIO_INTR_DISABLE,
    };

    unsafe { gpio_config(&config).as_result() }
}


pub fn configure_pin_as_output_raw(gpio: gpio_num_t) {
    log!(TAG, "configure pin for blinky: {:?}", gpio);
    unsafe {
        core::ptr::write_volatile(gpio_reg::GPIO_ENABLE_W1TS_REG as *mut _, 0x1 << (gpio as u32));

        let gpio_funcx_out_sel_cfg: u32 = gpio_reg::GPIO_FUNCX_OUT_BASE + ((gpio as u32) * 4);
        core::ptr::write_volatile(gpio_funcx_out_sel_cfg as *mut _, 0x100);
    }
}


pub fn set_led(gpio: gpio_num_t, val: bool) -> Result<(), EspError > {
    unsafe { gpio_set_level(gpio, val as u32).as_result() }
}


pub fn set_led_raw(gpio: gpio_num_t, val: bool) {
    if val {
        unsafe {
            core::ptr::write_volatile(gpio_reg::GPIO_OUT_W1TS_REG as *mut u32, 0x1 << (gpio as u8));
        }
    } else {
       unsafe {
            core::ptr::write_volatile(gpio_reg::GPIO_OUT_W1TC_REG as *mut u32, 0x1 << (gpio as u8));
        }
    }
}


pub fn delay(clocks: u32) {
    let target = get_ccount() + clocks;
    loop {
        if get_ccount() > target {
            break;
        }
    }
}


fn get_ccount() -> u32 {
    let x: u32;
    unsafe {
        asm!("rsr.ccount a2" : "={a2}" (x))
    };
    x
}
